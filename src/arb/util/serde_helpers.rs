use serde::{Deserialize, Deserializer, Serialize, Serializer};

pub mod u128_as_string {
    use super::*;

    pub fn serialize<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse::<u128>()
            .map_err(|_| serde::de::Error::custom("Failed to parse u128 from string"))
    }
}

pub mod byte_array_57 {
    use super::*;

    pub fn serialize<S>(value: &[u8; 57], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 57], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        if vec.len() != 57 {
            return Err(serde::de::Error::custom(format!(
                "Expected array of length 57, got {}",
                vec.len()
            )));
        }
        let mut arr = [0u8; 57];
        arr.copy_from_slice(&vec);
        Ok(arr)
    }
}

pub mod byte_array_159 {
    use super::*;

    pub fn serialize<S>(value: &[u8; 159], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.as_slice().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[u8; 159], D::Error>
    where
        D: Deserializer<'de>,
    {
        let vec = Vec::<u8>::deserialize(deserializer)?;
        if vec.len() != 159 {
            return Err(serde::de::Error::custom(format!(
                "Expected array of length 159, got {}",
                vec.len()
            )));
        }
        let mut arr = [0u8; 159];
        arr.copy_from_slice(&vec);
        Ok(arr)
    }
}
