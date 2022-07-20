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
