//! General utilities for the library

/// General wrapper for tracing function calls
///
/// Takes a mandatory function name, as well as optionally a format string and one or
/// more arguments. This allows tracing function calls with the called arguments. E.g.
/// ```ignore
/// fn f_name(foo, bar) {
///     trace_call!("f_name","called with {:?}, {:?}", foo, bar);
///     ...
/// }
/// ```
/// or
/// ```ignore
/// fn g() {
///     trace_call!("g");
/// }
/// ```
/// which will simply log `g called`.
#[macro_export]
macro_rules! trace_call {
    ($fn:literal) => {
        let _span = trace_span!($fn);
        let _guard = _span.enter();
        trace!("called");
    };
    ($fn:literal, $estr:literal) => {
        let _span = trace_span!($fn);
        let _guard = _span.enter();
        trace!($estr);
    };
    ($fn:literal, $estr:literal, $($arg:ident),+) => {
        let _span = trace_span!($fn);
        let _guard = _span.enter();
        trace!($estr, $($arg),+);
    };
}

#[cfg(test)]
pub(crate) mod test_utils {
    //! Common utilities for several test modules

    use crate::types::*;

    use tracing::Level;

    pub(crate) fn common_repo_def() -> RepoCommon {
        RepoCommon {
            passwd: "test".to_string(),
        }
    }

    pub fn local_repo_def(name: &str) -> Repo {
        Repo::Local {
            data: LocalRepo {
                path: name.to_string(),
                common: common_repo_def(),
            },
        }
    }

    pub(crate) fn aws_key_def() -> AWSKey {
        AWSKey {
            id: "the_id".to_string(),
            secret: "the_secret".to_string(),
        }
    }

    pub fn s3_repo_def() -> Repo {
        Repo::S3 {
            data: S3Repo {
                bucket: "foo".to_string(),
                url: "example.org".to_string(),
                region: Region::EuWest1,
                path: Some("bar".to_string()),
                key: aws_key_def(),
                common: common_repo_def(),
            },
        }
    }

    pub fn log_init() {
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::TRACE)
            .with_test_writer()
            .try_init();
    }
}
