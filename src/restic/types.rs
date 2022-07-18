use std::process::{Command, Output};
use tracing::trace;

// The WrappedCall trait and its implementation for ResticCall
//
// The motivation for this even existing is to allow mocking of the call, for testing
// reasons. I can pretend it also makes the API a _little_ cleaner, but that's
// absolutely a post-rationalization.
//
// It is what it is.

pub trait WrappedCall {
    fn invoke(&mut self) -> Result<Output, std::io::Error>;
    fn arg(&mut self, arg: String) -> &mut Self;
    fn env(&mut self, key: String, value: String) -> &mut Self;
}

#[derive(Debug)]
pub struct ResticCall {
    cmd: Command,
}

impl ResticCall {
    pub fn new() -> ResticCall {
        ResticCall {
            cmd: Command::new("restic"),
        }
    }
}

impl WrappedCall for ResticCall {
    fn invoke(&mut self) -> Result<Output, std::io::Error> {
        trace!("Invoking {:?}", self.cmd);
        self.cmd.output()
    }
    fn arg(&mut self, arg: String) -> &mut Self {
        trace!("Adding argument {:?}", arg);
        self.cmd.arg(arg);
        self
    }
    fn env(&mut self, key: String, val: String) -> &mut Self {
        trace!("Adding envvar {:?} = {:?}", key, val);
        self.cmd.env(key, val);
        self
    }
}

impl Default for ResticCall {
    fn default() -> Self {
        Self::new()
    }
}

// API data structs.
//
// These need to be re-exported, since upper-level modules need to use them to
// communicate with the API.

#[derive(Debug)]
pub struct RepoCommon {
    pub passwd: String,
}

#[derive(Debug)]
pub(crate) struct AWSKey {
    pub(crate) id: String,
    pub(crate) secret: String,
}

#[derive(Debug)]
pub struct LocalRepo {
    pub path: String,
    pub common: RepoCommon,
}

#[derive(Debug)]
pub struct S3Repo {
    pub url: String,
    pub bucket: String,
    pub region: String,
    pub path: Option<String>,
    pub(crate) key: AWSKey,
    pub common: RepoCommon,
}

#[derive(Debug)]
pub enum Repo {
    Local { data: LocalRepo },
    S3 { data: S3Repo },
}
