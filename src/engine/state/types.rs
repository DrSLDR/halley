//! Types belonging to the statefile processor

use std::{collections::HashMap, error::Error, fmt::Display, path::PathBuf, str::FromStr};

use anyhow::anyhow;
use serde::{de, de::Visitor, Deserialize, Serialize};

use crate::config::Config;

/// Top-level statefile representation
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct State {
    pub(crate) version: u32,
    pub(crate) states: HashMap<String, RepoState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            version: 1,
            states: HashMap::new(),
        }
    }
}

/// Representation of an individual repository state
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct RepoState {
    pub(crate) time: u64,
    pub(crate) digest: HexDigest,
}

impl Default for RepoState {
    fn default() -> Self {
        Self {
            time: 0,
            digest: HexDigest::from_str("c0ffee").unwrap(),
        }
    }
}

/// Hexadecimal digest representation
///
/// `String` isn't sufficiently narrow, and we'd rather just handle the `Vec<u8>` that
/// `dasher` will be producing internally, anyway.
///
/// May also be an excuse to learn how to write a proper `serde` implementation.
#[derive(PartialEq, Eq)]
pub(crate) struct HexDigest {
    data: Vec<u8>,
}

impl HexDigest {
    /// Creates a Digest from a given vector
    pub fn new(data: Vec<u8>) -> Self {
        Self { data }
    }

    /// Returns a reference to the internal vector
    pub fn get(&self) -> &Vec<u8> {
        &self.data
    }

    /// Converts the `HexDigest` back into a `String`
    pub fn to_string(&self) -> String {
        self.data
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }

    /// Helper function used in string decoding
    fn decode(b: u8, index: usize) -> anyhow::Result<u8> {
        match b {
            b'A'..=b'F' => Ok(b - b'A' + 10),
            b'a'..=b'f' => Ok(b - b'a' + 10),
            b'0'..=b'9' => Ok(b - b'0'),
            _ => Err(anyhow!("Illegal hex character {:?} at {:?}", b, index)),
        }
    }
}

impl Display for HexDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_string())
    }
}

impl std::fmt::Debug for HexDigest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "\"{}\"", self.to_string())
    }
}

impl FromStr for HexDigest {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> anyhow::Result<Self> {
        let hex = s.as_bytes();
        if hex.len() % 2 != 0 {
            return Err(anyhow!("Odd length of hex digest"));
        }

        let vector: Vec<u8> = hex
            .chunks(2)
            .enumerate()
            .map(|(i, pair)| {
                Ok(Self::decode(pair[0], 2 * i)? << 4 | Self::decode(pair[1], 2 * i + 1)?)
            })
            .collect::<Result<Vec<u8>, Self::Err>>()?;

        Ok(Self::new(vector))
    }
}

impl Serialize for HexDigest {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for HexDigest {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        deserializer.deserialize_string(HexDigestVisitor)
    }
}

struct HexDigestVisitor;

impl<'de> Visitor<'de> for HexDigestVisitor {
    type Value = HexDigest;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("a hexadecimal-encoded string")
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match HexDigest::from_str(v) {
            Ok(h) => Ok(h),
            Err(e) => Err(E::custom(e.to_string())),
        }
    }

    fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        self.visit_str(&v)
    }
}

/// Struct used to call the `check` function
///
/// Looks a lot like `RunSpec`, but isn't really
#[derive(Debug)]
pub(crate) struct CheckArgs<'a> {
    pub statefile: PathBuf,
    pub config: &'a Config,
    pub dry: bool,
    pub specific_repo: &'a Option<String>,
}

/// Return Enum from the `check` function
#[derive(Debug)]
pub(crate) enum StateStatus {
    NothingToDo,
    NextRepo(String),
}

/// Special error types related to state management
#[derive(Debug)]
pub(crate) enum StateError {
    Glob(glob::GlobError),
    IO(std::io::Error),
    TOMLD(toml::de::Error),
    TOMLS(toml::ser::Error),
    Dasher(dasher::HashError),
    Internal(ErrorKind),
}

impl Display for StateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            StateError::Internal(ref err) => {
                write!(f, "An internal error occurred: {:?}", err)
            }
            StateError::Glob(ref err) => err.fmt(f),
            StateError::IO(ref err) => err.fmt(f),
            StateError::TOMLD(ref err) => err.fmt(f),
            StateError::TOMLS(ref err) => err.fmt(f),
            StateError::Dasher(ref err) => err.fmt(f),
        }
    }
}

impl From<std::io::Error> for StateError {
    fn from(err: std::io::Error) -> StateError {
        StateError::IO(err)
    }
}

impl From<toml::de::Error> for StateError {
    fn from(err: toml::de::Error) -> StateError {
        StateError::TOMLD(err)
    }
}

impl From<toml::ser::Error> for StateError {
    fn from(err: toml::ser::Error) -> StateError {
        StateError::TOMLS(err)
    }
}

impl From<glob::GlobError> for StateError {
    fn from(err: glob::GlobError) -> StateError {
        StateError::Glob(err)
    }
}

impl From<dasher::HashError> for StateError {
    fn from(err: dasher::HashError) -> StateError {
        StateError::Dasher(err)
    }
}

impl Error for StateError {}

/// Error flavors
#[derive(Clone, Debug)]
pub(crate) enum ErrorKind {
    StateFileDoesNotExist,
    BadPath,
}
