//! S3 interface
//!
//! This module handles talking to an S3 endpoint, in order to check whether a
//! repository exists, as well as ferrying objects in and out of the Glacier storage
//! class.

#[cfg(test)]
mod tests;

use crate::trace_call;
use crate::types::{AWSKey, RepoCommon, S3Repo};

use std::str::FromStr;
use std::string::ToString;
use std::thread;
use std::time::{Duration, Instant};

use async_recursion::async_recursion;
use rusoto_core::{credential, Client, RusotoError};
use rusoto_s3::{
    CopyObjectRequest, HeadBucketRequest, HeadObjectRequest, ListObjectsV2Request,
    RestoreObjectRequest, S3Client, S3,
};
use tokio;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, trace, trace_span, warn};

/// Stateful struct containing the `S3Client` and relevant helper data
pub(crate) struct S3Handler {
    url: String,
    bucket: String,
    prefix: Option<String>,
    _repo: S3Repo,
    alloc_size: usize,
    hold_time: Duration,
    concurrent_tasks: usize,
    retry_count: usize,
    retry_wait: Duration,
    client: S3Client,
}

impl std::fmt::Debug for S3Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("S3Handler").field("url", &self.url).finish()
    }
}

impl Clone for S3Handler {
    fn clone(&self) -> Self {
        Self::new(S3Repo {
            url: self._repo.url.clone(),
            bucket: self.bucket.clone(),
            region: self._repo.region.clone(),
            path: self.prefix.clone(),
            key: AWSKey {
                id: self._repo.key.id.clone(),
                secret: self._repo.key.secret.clone(),
            },
            common: RepoCommon {
                passwd: self._repo.common.passwd.clone(),
            },
        })
    }
}

/// Defines the storage classes we can handle
#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) enum StorageClass {
    STANDARD,
    GLACIER,
}

impl FromStr for StorageClass {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<StorageClass, Self::Err> {
        match s {
            "STANDARD" => Ok(StorageClass::STANDARD),
            "GLACIER" => Ok(StorageClass::GLACIER),
            _ => Err(anyhow::Error::msg(format!(
                "StorageClass string {} could not be parsed",
                s
            ))),
        }
    }
}

impl ToString for StorageClass {
    fn to_string(&self) -> String {
        match self {
            Self::STANDARD => "STANDARD".to_string(),
            Self::GLACIER => "GLACIER".to_string(),
        }
    }
}

/// Defines a object record, so we can track storage class straight away
#[derive(Debug, Clone)]
pub(crate) struct Object {
    pub key: String,
    pub class: StorageClass,
}

// Be mindful that sometimes requests just _fail_ on their own - consider handling
// timeouts in a way that makes sense, rather than just failing them.

impl S3Handler {
    /// Creates a new [`S3Handler`] which can be used to communicate with a given repository.
    pub fn new(repo: S3Repo) -> S3Handler {
        trace_call!("new", "called with {:?}", repo);
        let client = S3Client::new_with_client(
            Client::new_with(
                credential::StaticProvider::new_minimal(
                    repo.key.id.clone(),
                    repo.key.secret.clone(),
                ),
                rusoto_core::HttpClient::new().unwrap(),
            ),
            repo.region.clone(),
        );
        S3Handler::new_with_client(repo, client)
    }

    /// Creates a new [`S3Handler`] with a given internal `S3Client`
    pub fn new_with_client(repo: S3Repo, client: S3Client) -> S3Handler {
        trace_call!("new_with_client", "called with {:?}", repo);
        S3Handler {
            url: repo.render_full_url(),
            bucket: repo.bucket.clone(),
            prefix: repo.path.clone(),
            alloc_size: {
                warn!("Still using hardcoded, default item vector capacity!");
                1024
            },
            hold_time: {
                warn!("Still using hardcoded, default hold time!");
                Duration::from_secs(15)
            },
            concurrent_tasks: {
                warn!("Still using hardcoded, default concurrent tasks count!");
                1
            },
            retry_count: {
                warn!("Still using hardcoded, default retry count!");
                5
            },
            retry_wait: {
                warn!("Still using hardcoded, default retry wait time!");
                Duration::from_secs(2)
            },
            client,
            _repo: repo,
        }
    }

    /// Calls the client, retrying if certain errors occur
    async fn call_retrying<'l, 'a, A, O, E>(
        &'l self,
        f: fn(
            &'l S3Client,
            A,
        ) -> std::pin::Pin<
            Box<dyn std::future::Future<Output = Result<O, RusotoError<E>>> + Send + 'a>,
        >,
        args: A,
    ) -> Result<O, RusotoError<E>> {
        for _ in 0..self.retry_count {
            match f(&self.client, args).await {
                Ok(o) => return Ok(o),
                Err(e) => unimplemented!(),
            }
        }
        unimplemented!()
    }

    /// Tests whether the related bucket exists
    pub async fn bucket_exists(&self) -> anyhow::Result<bool> {
        trace_call!("bucket_exists", "called on {:?}", self);
        let response = self
            .client
            .head_bucket(HeadBucketRequest {
                bucket: self.bucket.clone(),
                expected_bucket_owner: None,
            })
            .await;
        match response {
            Ok(()) => {
                debug!("Bucket {} exists", &self.bucket);
                Ok(true)
            }
            Err(e) => {
                error!("Checking for bucket existence failed! See debug log for more details.");
                debug!("{:?}", e);
                Ok(false)
            }
        }
    }

    /// Actually calls S3 to gather all the objects in the bucket, possibly in the given
    /// path.
    ///
    /// This is an internal helper function which is async recursive, which is all sorts
    /// of fun. This is due to the (potential) necessity of handling a
    /// `continuation_token`, since a call will only return up to 1000 items.
    ///
    /// The data itself is added to the mutable `store` vector passed in.
    #[async_recursion]
    async fn list_objects(
        &self,
        store: &mut Vec<Object>,
        token: Option<String>,
    ) -> anyhow::Result<()> {
        trace_call!(
            "list_objects",
            "called with store {:?}, token {:?}",
            store,
            token
        );

        let mut r = ListObjectsV2Request::default();
        r.bucket = self.bucket.clone();
        r.continuation_token = token;
        r.prefix = match &self.prefix {
            Some(s) => Some(s.clone()),
            None => None,
        };

        match self.client.list_objects_v2(r).await {
            Ok(data) => match data.contents {
                Some(contents) => {
                    for object in contents {
                        store.push(Object {
                            key: object.key.clone().unwrap(),
                            class: match object.storage_class {
                                Some(class) => class.parse::<StorageClass>()?,
                                None => {
                                    warn!(
                                        "Failed to get any storage class for {}, assuming STANDARD",
                                        object.key.unwrap()
                                    );
                                    "STANDARD".parse::<StorageClass>()?
                                }
                            },
                        })
                    }
                    match data.next_continuation_token {
                        Some(token) => self.list_objects(store, Some(token)).await,
                        None => Ok(()),
                    }
                }
                None => {
                    match data.continuation_token {
                        Some(_) => info!("Object listing call on {:?} returned nothing\nAssuming it just ran out of items", self),
                        None => warn!("Object listing call on {:?} returned nothing", self),
                    }
                    Ok(())
                }
            },
            Err(e) => {
                error!("Failed to list items! See debug log for more details.");
                debug!("{:?}", e);
                Err(anyhow::Error::new(e))
            }
        }
    }

    /// Collects a list of all keys in the given bucket and path
    pub async fn list_all_objects(&self) -> anyhow::Result<Vec<Object>> {
        trace_call!("list_all_objects", "called on {:?}", self);
        let mut items: Vec<Object> = Vec::with_capacity(self.alloc_size);
        self.list_objects(&mut items, None).await?;

        info!("Listed {} items in {:?}", items.len(), self);
        if items.is_empty() {
            warn!("Listed no items in {:?}!", self);
        }
        debug!("Collected items {:#?}", items);
        debug!(
            "Vector utilization at {:.1}% of initialization ({})",
            { (items.len() as f32 / self.alloc_size as f32) * 100.0 },
            self.alloc_size
        );

        Ok(items)
    }

    /// Gets the [`StorageClass`] of the given key
    pub async fn get_storage_class(&self, key: String) -> anyhow::Result<StorageClass> {
        trace_call!("get_storage_class", "called with {:?}", key);

        let mut r = HeadObjectRequest::default();
        r.bucket = self.bucket.clone();
        r.key = key.clone();

        match self.client.head_object(r).await {
            Ok(head) => match head.storage_class {
                Some(class) => Ok(class.parse::<StorageClass>()?),
                None => {
                    warn!(
                        "Failed to get any storage class for {}, assuming STANDARD",
                        key
                    );
                    Ok("STANDARD".parse::<StorageClass>()?)
                }
            },
            Err(e) => {
                error!("Could not head object! See debug log for more details.");
                debug!("{:?}", e);
                Err(anyhow::Error::new(e))
            }
        }
    }

    /// Restores an object from [`GLACIER`] to [`STANDARD`]
    ///
    /// [`GLACIER`]: StorageClass::GLACIER
    /// [`STANDARD`]: StorageClass::STANDARD
    pub async fn restore_object(&self, key: String) -> anyhow::Result<()> {
        trace_call!("restore_object", "called with key {:?}", key);

        let mut r = RestoreObjectRequest::default();
        r.bucket = self.bucket.clone();
        r.key = key.clone();

        match self.client.restore_object(r).await {
            Ok(_) => {
                debug!("Requested {} be restored", key);
                Ok(())
            }
            Err(e) => {
                error!("Failed to restore object! See debug log for details.");
                debug!("{:?}", e);
                Err(anyhow::Error::new(e))
            }
        }
    }

    /// Copies an object from [`STANDARD`] to [`GLACIER`]
    ///
    /// [`GLACIER`]: StorageClass::GLACIER
    /// [`STANDARD`]: StorageClass::STANDARD
    pub async fn archive_object(&self, key: String) -> anyhow::Result<()> {
        trace_call!("archive_object", "called with key {:?}", key);
        self.nospan_archive_object(key).await
    }

    /// [`archive_object`] without span logging
    async fn nospan_archive_object(&self, key: String) -> anyhow::Result<()> {
        let mut r = CopyObjectRequest::default();
        r.bucket = self.bucket.clone();
        r.copy_source = format!("{}/{}", self.bucket.clone(), key.clone());
        r.key = key.clone();
        r.storage_class = Some(StorageClass::GLACIER.to_string());

        match self.client.copy_object(r).await {
            Ok(_) => {
                debug!("Requested {} be archived", key);
                Ok(())
            }
            Err(e) => {
                error!("Failed to copy object! See debug log for more details.");
                debug!("{:?}", e);
                Err(anyhow::Error::new(e))
            }
        }
    }

    /// Enumerates all objects and requests that they be restored
    ///
    /// The function guarantees that only the objects not already restored will be
    /// queried to be restored. If all objects are already restored, then nothing will
    /// be done.
    ///
    /// Returns a list of objects that has been queried, which may be empty.
    pub async fn restore_all_objects(&self) -> anyhow::Result<Vec<Object>> {
        trace_call!("restore_all_objects");
        let start = Instant::now();

        let mut objects = self.list_all_objects().await?;
        objects.retain(|o| o.class == StorageClass::GLACIER);

        for object in objects.iter() {
            self.restore_object(object.key.clone()).await?;
        }

        let duration = start.elapsed();

        info!(
            "Requested restoration of {} objects in {:?}",
            objects.len(),
            duration
        );

        Ok(objects)
    }

    /// Enumerates all objects and requests that they be archived
    ///
    /// The function guarantees that only the objects not already archived will be
    /// queried to be archived. If all objects are already archived, then nothing will
    /// be done.
    ///
    /// Returns a list of objects that has been queried, which may be empty.
    pub async fn archive_all_objects(&self) -> anyhow::Result<Vec<Object>> {
        trace_call!("archive_all_objects");
        let start = Instant::now();

        let mut objects = self.list_all_objects().await?;
        // objects.retain(|o| o.class != StorageClass::GLACIER);

        let handles = (0..self.concurrent_tasks)
            .into_iter()
            .map(|i: usize| {
                debug!(
                    "Creating object iterator {} of {}, starting from index {}",
                    i + 1,
                    self.concurrent_tasks,
                    i
                );
                let mut iter = objects.iter();
                for _ in 0..i {
                    iter.next();
                }
                debug!(
                    "Creating subset vector from step size {}",
                    self.concurrent_tasks
                );
                let objects_subset: Vec<Object> = iter
                    .step_by(self.concurrent_tasks)
                    .map(|o| o.clone())
                    .collect();
                debug!("Got subset vector of length {}", objects_subset.len());

                let h = self.clone();
                tokio::spawn(async move {
                    for object in objects_subset {
                        h.nospan_archive_object(object.key)
                            .await
                            .expect("Failed a parallel archive task");
                    }
                })
            })
            .collect::<Vec<JoinHandle<()>>>();

        for handle in handles {
            handle.await?
        }

        let duration = start.elapsed();

        info!(
            "Requested archival of {} objects in {:?}",
            objects.len(),
            duration
        );

        Ok(objects)
    }

    /// Enumerates all objects and requests that they be restored. This function will
    /// not return until all objects have been restored.
    pub async fn restore_all_objects_blocking(&self) -> anyhow::Result<()> {
        trace_call!("restore_all_objects_blocking");

        let mut objects = self.restore_all_objects().await?;

        if objects.is_empty() {
            info!("No objects were in need of restoration, no need to block");
            return Ok(());
        }

        let start = Instant::now();
        let count = objects.len();
        debug!("Stepping into restoration blocking loop");
        'blocking: loop {
            debug!("Sleeping for {:?}", self.hold_time);
            thread::sleep(self.hold_time);

            'inner: loop {
                match objects.pop() {
                    Some(o) => match self.get_storage_class(o.key.clone()).await? {
                        StorageClass::STANDARD => debug!("{:?} successfully restored", o.key),
                        StorageClass::GLACIER => {
                            debug!("{:?} still archived, placing it back in the stack", o.key);
                            objects.push(o);
                            break 'inner;
                        }
                    },
                    None => {
                        debug!("Reached end of object list, breaking blocking loop");
                        break 'blocking;
                    }
                }
            }
        }
        let duration = start.elapsed();
        info!("Restored {} objects in {:?}", count, duration);

        Ok(())
    }
}
