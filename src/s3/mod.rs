//! S3 interface
//!
//! This module handles talking to an S3 endpoint, in order to check whether a
//! repository exists, as well as ferrying objects in and out of the Glacier storage
//! class.

#[cfg(test)]
mod tests;

use crate::trace_call;
use crate::types::S3Repo;

use std::str::FromStr;
use std::string::ToString;

use async_recursion::async_recursion;
use rusoto_core::{credential, Client};
use rusoto_s3::{
    CopyObjectRequest, HeadBucketRequest, HeadObjectRequest, ListObjectsV2Request,
    RestoreObjectRequest, S3Client, S3,
};
use tracing::{debug, error, info, trace, trace_span, warn};

/// Stateful struct containing the `S3Client` and relevant helper data
pub(crate) struct S3Handler {
    url: String,
    bucket: String,
    prefix: Option<String>,
    alloc_size: usize,
    client: S3Client,
}

impl std::fmt::Debug for S3Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("S3Handler").field("url", &self.url).finish()
    }
}

/// Defines the storage classes we can handle
#[derive(Debug)]
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
#[derive(Debug)]
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
        S3Handler {
            url: repo.render_full_url(),
            bucket: repo.bucket,
            prefix: repo.path,
            alloc_size: {
                warn!("Still using hardcoded, default item vector capacity!");
                1024
            },
            client: S3Client::new_with_client(
                Client::new_with(
                    credential::StaticProvider::new_minimal(repo.key.id, repo.key.secret),
                    rusoto_core::HttpClient::new().unwrap(),
                ),
                repo.region,
            ),
        }
    }

    /// Tests whether the related bucket exists
    pub async fn bucket_exists(&self) -> bool {
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
                true
            }
            Err(e) => {
                error!("Checking for bucket existence failed! See debug log for more details.");
                debug!("{:?}", e);
                false
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
    pub async fn list_all_items(&self) -> anyhow::Result<Vec<Object>> {
        trace_call!("list_all_items", "called on {:?}", self);
        let mut items: Vec<Object> = Vec::with_capacity(self.alloc_size);
        self.list_objects(&mut items, None).await?;

        info!("Listed {} items in {:?}", items.len(), self);
        if items.is_empty() {
            warn!("Listed no items in {:?}!", self);
        }
        debug!("Collected items {:#?}", items);
        debug!(
            "Vector utilization at {}% of initialization ({})",
            { items.len() / self.alloc_size },
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

        let mut r = CopyObjectRequest::default();
        r.bucket = self.bucket.clone();
        r.copy_source = format!("{}/{}", self.bucket.clone(), key.clone());
        r.key = key.clone();
        r.storage_class = Some(StorageClass::STANDARD.to_string());

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
}
