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

use async_recursion::async_recursion;
use rusoto_core::{credential, Client};
use rusoto_s3::{HeadBucketRequest, HeadObjectRequest, ListObjectsV2Request, S3Client, S3};
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
                bucket: self.bucket.to_owned(),
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
        store: &mut Vec<String>,
        token: Option<String>,
    ) -> anyhow::Result<()> {
        trace_call!(
            "list_objects",
            "called with store {:?}, token {:?}",
            store,
            token
        );
        match self
            .client
            .list_objects_v2(ListObjectsV2Request {
                bucket: self.bucket.to_owned(),
                continuation_token: token,
                delimiter: None,
                encoding_type: None,
                expected_bucket_owner: None,
                fetch_owner: None,
                max_keys: None,
                prefix: match &self.prefix {
                    Some(s) => Some(s.to_owned()),
                    None => None,
                },
                request_payer: None,
                start_after: None,
            })
            .await
        {
            Ok(data) => match data.contents {
                Some(contents) => {
                    for object in contents {
                        store.push(object.key.unwrap())
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
    pub async fn list_all_items(&self) -> anyhow::Result<Vec<String>> {
        trace_call!("list_all_items", "called on {:?}", self);
        let mut items: Vec<String> = Vec::with_capacity(self.alloc_size);
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
        match self
            .client
            .head_object(HeadObjectRequest {
                bucket: self.bucket.to_owned(),
                expected_bucket_owner: None,
                if_match: None,
                if_modified_since: None,
                if_none_match: None,
                if_unmodified_since: None,
                key: key.to_owned(),
                part_number: None,
                range: None,
                request_payer: None,
                sse_customer_algorithm: None,
                sse_customer_key: None,
                sse_customer_key_md5: None,
                version_id: None,
            })
            .await
        {
            Ok(head) => {
                debug!("{:#?}", head);
                match head.storage_class {
                    Some(class) => Ok(class.parse::<StorageClass>()?),
                    None => {
                        warn!(
                            "Failed to get any storage class for {}, assuming STANDARD",
                            key
                        );
                        Ok("STANDARD".parse::<StorageClass>()?)
                    }
                }
            }
            Err(e) => {
                error!("Could not head object! See debug log for more details.");
                debug!("{:?}", e);
                Err(anyhow::Error::new(e))
            }
        }
    }

    /// Copies an object to a given storage class
    ///
    /// Another internal helper function
    async fn change_storage_class(
        &self,
        key: String,
        to_class: StorageClass,
    ) -> anyhow::Result<()> {
        trace_call!(
            "change_storage_class",
            "called with key {:?}, class {:?}",
            key,
            to_class
        );

        unimplemented!()
    }

    /// Copies an object from [`GLACIER`] to [`STANDARD`]
    ///
    /// [`GLACIER`]: StorageClass::GLACIER
    /// [`STANDARD`]: StorageClass::STANDARD
    pub async fn restore_object(&self, key: String) -> anyhow::Result<()> {
        trace_call!("restore_object", "called with key {:?}", key);

        unimplemented!()
    }

    /// Copies an object from [`STANDARD`] to [`GLACIER`]
    ///
    /// [`GLACIER`]: StorageClass::GLACIER
    /// [`STANDARD`]: StorageClass::STANDARD
    pub async fn archive_object(&self, key: String) -> anyhow::Result<()> {
        trace_call!("archive_object", "called with key {:?}", key);

        unimplemented!()
    }
}
