use super::*;
use crate::types::*;
use crate::util::test_utils::*;

use rusoto_mock::*;

macro_rules! get_s3_repo {
    () => {
        match s3_repo_def() {
            Repo::S3 { data } => data,
            _ => unimplemented!(),
        }
    };
}

#[test]
fn spawn_handler() {
    let repo = get_s3_repo!();
    let _h: S3Handler = S3Handler::new(repo);
}

/// HTTP response codes that MUST cause a retry
macro_rules! retry_http {
    () => {
        [408, 429, 500, 502, 503, 504]
    };
}

/// HTTP response codes that MUST cause a failure
macro_rules! fail_http {
    () => {
        [400]
    };
}

macro_rules! make_s3_client {
    ($rd:expr, $reg:expr) => {
        S3Client::new_with($rd, MockCredentialsProvider, $reg.clone())
    };
    ($reg:expr) => {
        make_s3_client!(MockRequestDispatcher::default(), $reg)
    };
}

macro_rules! s3h {
    ($rd:expr, $repo:expr) => {
        S3Handler::new_with_client($repo, make_s3_client!($rd, $repo.region))
    };
    ($rd:expr) => {
        s3h!($rd, get_s3_repo!())
    };
    () => {
        s3h!(MockRequestDispatcher::default(), get_s3_repo!())
    };
}

#[tokio::test]
async fn bucket_exists() {
    let h = s3h!();
    assert!(h.bucket_exists().await.unwrap());

    let rd = MockRequestDispatcher::with_status(403);
    let h = s3h!(rd);
    assert!(h.bucket_exists().await.is_err());

    let rd = MockRequestDispatcher::with_status(404);
    let h = s3h!(rd);
    assert!(!h.bucket_exists().await.unwrap());

    for code in retry_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.bucket_exists().await.unwrap())
    }

    for code in fail_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.bucket_exists().await.is_err());
    }
}

#[tokio::test]
async fn list_all_objects() {
    let h = s3h!();
    let v = h.list_all_objects().await;
    assert!(v.is_ok());
    assert!(v.unwrap().is_empty());

    for code in retry_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.list_all_objects().await.is_ok())
    }

    for code in fail_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.list_all_objects().await.is_err());
    }

    let body = "<?xml version='1.0' encoding='UTF-8'?>
    <ListBucketResult
        xmlns=\"http://s3.amazonaws.com/doc/2006-03-01/\">
        <Name>testbucket-2</Name>
        <Prefix/>
        <Delimiter/>
        <MaxKeys>1000</MaxKeys>
        <IsTruncated>false</IsTruncated>
        <KeyCount>910</KeyCount>
        <Contents>
            <ETag>\"d1b45e489539fdd080b6a8bd51ac8846\"</ETag>
            <Key>bar/config</Key>
            <LastModified>2022-07-28T21:39:20.000Z</LastModified>
            <Size>155</Size>
            <StorageClass>GLACIER</StorageClass>
        </Contents>
        <Contents>
            <ETag>\"031dbfb83084336de714837a1884bc90\"</ETag>
            <Key>bar/kimchi</Key>
            <LastModified>2022-07-28T21:39:28.000Z</LastModified>
            <Size>450</Size>
        </Contents>
    </ListBucketResult>";
    let rd = MockRequestDispatcher::default();
    let rd = rd.with_body(body);
    let h = s3h!(rd);
    let r = h.list_all_objects().await;
    assert!(r.is_ok());
    assert!(r.unwrap().len() == 2);
}
