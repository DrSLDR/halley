//! S3 interface
//!
//! This module handles talking to an S3 endpoint, in order to check whether a
//! repository exists, as well as ferrying objects in and out of the Glacier storage
//! class.

#[cfg(test)]
mod tests;

use crate::trace_call;
use crate::types::S3Repo;

use async_recursion::async_recursion;
use rusoto_core::{credential, Client};
use rusoto_s3::{HeadBucketRequest, ListObjectsV2Request, S3Client, S3};
use tracing::{debug, error, trace, trace_span, warn};

pub(crate) struct S3Handler {
    url: String,
    bucket: String,
    prefix: Option<String>,
    client: S3Client,
}

impl std::fmt::Debug for S3Handler {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.debug_struct("S3Handler").field("url", &self.url).finish()
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
            client: S3Client::new_with_client(
                Client::new_with(
                    credential::StaticProvider::new_minimal(repo.key.id, repo.key.secret),
                    rusoto_core::HttpClient::new().unwrap(),
                ),
                repo.region,
            ),
        }
    }

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
                max_keys: Some(1),
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
                        None => Ok(())
                    }
                }
                None => {
                    warn!("Object listing call on {:?} returned nothing", self);
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

    pub async fn list_all_items(&self) -> anyhow::Result<Vec<String>> {
        trace_call!("list_all_items", "called on {:?}", self);
        let mut items: Vec<String> = Vec::with_capacity(1024);
        warn!("Still using hardcoded, default item vector capacity!");
        let token = self.list_objects(&mut items, None).await?;

        debug!("Gathered items {:#?}", items);

        Ok(vec![])
    }
}
