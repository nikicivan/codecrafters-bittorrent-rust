use serde::de::{self, Deserialize, Deserializer, Visitor};
use serde::ser::{Serialize, Serializer};
use std::fmt;

#[derive(Clone, Debug)]
pub struct Hashes(pub Vec<[u8; 20]>);

struct HashesVisitor;

impl<'de> Visitor<'de> for HashesVisitor {
    type Value = Hashes;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("hashes vec with length of multiple to 20")
    }

    fn visit_bytes<E>(self, v: &[u8]) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        if v.len() % 20 != 0 {
            return Err(E::custom(format!("Invalid length of {}", v.len())));
        }

        Ok(Hashes(
            v.chunks(20)
                .map(|slice_20| slice_20.try_into().expect("length should be 20"))
                .collect(),
        ))
    }
}

impl<'de> Deserialize<'de> for Hashes {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_bytes(HashesVisitor)
    }
}

impl Serialize for Hashes {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(&self.0.concat())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_hashes_deserialize() {
        let hashes = Hashes(vec![[0u8; 20]; 3]);
        println!("{:?}", hashes);
        // assert_eq!(hashes,deserialized);
    }
}
