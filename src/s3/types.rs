use std::str::FromStr;
use std::string::ToString;

/// Defines the storage classes we can handle
#[derive(Debug, PartialEq, Eq, Clone)]
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
#[derive(Debug, Clone)]
pub(crate) struct Object {
    pub key: String,
    pub class: StorageClass,
}
