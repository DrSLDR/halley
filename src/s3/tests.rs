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
    log_init();
    let repo = get_s3_repo!();
    let _h: S3Handler = S3Handler::new(repo);
}

/// HTTP response codes that MUST cause a retry
macro_rules! retry_http {
    () => {
        [408, 429, 500, 502, 504]
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
    ($rd:expr, $repo:expr) => {{
        log_init();
        S3Handler::new_with_client($repo, make_s3_client!($rd, $repo.region))
    }};
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

#[tokio::test]
async fn get_storage_class() {
    let h = s3h!();
    let v = h.get_storage_class("foo".to_string()).await;
    assert!(v.is_ok());
    assert!(v.unwrap() == StorageClass::STANDARD);

    let rd = MockRequestDispatcher::default().with_header("x-amz-storage-class", "GLACIER");
    let h = s3h!(rd);
    let v = h.get_storage_class("foo".to_string()).await;
    assert!(v.is_ok());
    assert!(v.unwrap() == StorageClass::GLACIER);

    let rd = MockRequestDispatcher::default().with_header("x-amz-storage-class", "STANDARD");
    let h = s3h!(rd);
    let v = h.get_storage_class("foo".to_string()).await;
    assert!(v.is_ok());
    assert!(v.unwrap() == StorageClass::STANDARD);

    let rd = MockRequestDispatcher::default().with_header("x-amz-storage-class", "lol");
    let h = s3h!(rd);
    assert!(h.get_storage_class("foo".to_string()).await.is_err());

    let rd = MockRequestDispatcher::with_status(403);
    let h = s3h!(rd);
    assert!(h.get_storage_class("foo".to_string()).await.is_err());

    let rd = MockRequestDispatcher::with_status(404);
    let h = s3h!(rd);
    assert!(h.get_storage_class("foo".to_string()).await.is_err());

    for code in retry_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.get_storage_class("foo".to_string()).await.is_ok())
    }

    for code in fail_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.get_storage_class("foo".to_string()).await.is_err());
    }
}

#[tokio::test]
async fn restore_object() {
    let h = s3h!();
    let v = h.restore_object("foo".to_string()).await;
    assert!(v.is_ok());
    assert!(v.unwrap() == ());

    let rd = MockRequestDispatcher::with_status(202);
    let h = s3h!(rd);
    let v = h.restore_object("foo".to_string()).await;
    assert!(v.is_ok());
    assert!(v.unwrap() == ());

    let rd = MockRequestDispatcher::with_status(409);
    let h = s3h!(rd);
    let v = h.restore_object("foo".to_string()).await;
    assert!(v.is_ok());
    assert!(v.unwrap() == ());

    let rd = MockRequestDispatcher::with_status(403);
    let h = s3h!(rd);
    assert!(h.restore_object("foo".to_string()).await.is_err());

    let rd = MockRequestDispatcher::with_status(404);
    let h = s3h!(rd);
    assert!(h.restore_object("foo".to_string()).await.is_err());

    for code in retry_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.restore_object("foo".to_string()).await.is_ok())
    }

    for code in fail_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.restore_object("foo".to_string()).await.is_err());
    }
}

#[tokio::test]
async fn archive_object() {
    let h = s3h!();
    let v = h.archive_object("foo".to_string()).await;
    assert!(v.is_ok());
    assert!(v.unwrap() == ());

    let rd = MockRequestDispatcher::with_status(412);
    let h = s3h!(rd);
    let v = h.archive_object("foo".to_string()).await;
    assert!(v.is_err());

    let rd = MockRequestDispatcher::with_status(403);
    let h = s3h!(rd);
    let v = h.archive_object("foo".to_string()).await;
    assert!(v.is_err());

    let rd = MockRequestDispatcher::with_status(404);
    let h = s3h!(rd);
    let v = h.archive_object("foo".to_string()).await;
    assert!(v.is_err());

    for code in retry_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.archive_object("foo".to_string()).await.is_ok())
    }

    for code in fail_http!() {
        let rd = MultipleMockRequestDispatcher::new([
            MockRequestDispatcher::with_status(code),
            MockRequestDispatcher::default(),
        ]);
        let h = s3h!(rd);
        assert!(h.archive_object("foo".to_string()).await.is_err());
    }
}

#[test]
fn logistic() {
    let mut h = s3h!();
    h.max_concurrent_tasks = 1;
    assert_eq!(h.logistic(0), 0);
    assert_eq!(h.logistic(1), 1);
    assert_eq!(h.logistic(8), 1);
    assert_eq!(h.logistic(511), 1);

    let h = s3h!();
    assert_eq!(h.logistic(0), 0);
    assert_eq!(h.logistic(1), 1);
    assert_eq!(h.logistic(30), 1);
    assert_eq!(h.logistic(200), 2);
    assert_eq!(h.logistic(1024), 8);
    assert_eq!(h.logistic(2100), 16);

    let mut h = s3h!();
    h.max_concurrent_tasks = 128;
    assert_eq!(h.logistic(0), 0);
    assert_eq!(h.logistic(1), 1);
    assert_eq!(h.logistic(3), 3);
}
