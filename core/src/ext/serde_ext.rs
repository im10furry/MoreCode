use serde::Serialize;
use serde::de::DeserializeOwned;

use crate::McError;

pub fn to_json_string<T>(value: &T) -> Result<String, McError>
where
    T: Serialize,
{
    serde_json::to_string(value).map_err(|error| McError::SerializationFailed {
        reason: error.to_string(),
    })
}

pub fn to_pretty_json_string<T>(value: &T) -> Result<String, McError>
where
    T: Serialize,
{
    serde_json::to_string_pretty(value).map_err(|error| McError::SerializationFailed {
        reason: error.to_string(),
    })
}

pub fn from_json_str<T>(input: &str) -> Result<T, McError>
where
    T: DeserializeOwned,
{
    serde_json::from_str(input).map_err(|error| McError::SerializationFailed {
        reason: error.to_string(),
    })
}

pub fn from_json_slice<T>(input: &[u8]) -> Result<T, McError>
where
    T: DeserializeOwned,
{
    serde_json::from_slice(input).map_err(|error| McError::SerializationFailed {
        reason: error.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};

    use super::{from_json_slice, from_json_str, to_json_string, to_pretty_json_string};

    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct Demo {
        name: String,
        count: u32,
    }

    #[test]
    fn serde_helpers_roundtrip_values() {
        let demo = Demo {
            name: "demo".into(),
            count: 2,
        };

        let compact = to_json_string(&demo).unwrap();
        let pretty = to_pretty_json_string(&demo).unwrap();
        assert!(pretty.contains('\n'));
        assert_eq!(from_json_str::<Demo>(&compact).unwrap(), demo);
        assert_eq!(from_json_slice::<Demo>(compact.as_bytes()).unwrap(), demo);
    }
}
