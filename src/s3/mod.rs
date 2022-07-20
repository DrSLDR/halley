//! S3 interface
//!
//! This module handles talking to an S3 endpoint, in order to check whether a
//! repository exists, as well as ferrying objects in and out of the Glacier storage
//! class.

use rusoto_core::Region;
use rusoto_s3::S3Client as Client;

fn init() -> Client {
    unimplemented!();
}
