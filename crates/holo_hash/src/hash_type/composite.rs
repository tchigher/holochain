use super::*;

#[cfg(all(test, feature = "serialized-bytes"))]
use holochain_serialized_bytes::prelude::*;

/// The AnyDht (composite) HashType
#[derive(
    Debug, Copy, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, serde::Deserialize, serde::Serialize,
)]
#[cfg_attr(all(test, feature = "serialized-bytes"), derive(SerializedBytes))]
#[serde(from = "AnyDhtSerial", into = "AnyDhtSerial")]
pub enum AnyDht {
    /// The hash of an Entry
    Entry,
    /// The hash of a Header
    Header,
}

impl HashType for AnyDht {
    fn get_prefix(self) -> &'static [u8] {
        match self {
            AnyDht::Entry => Entry::new().get_prefix(),
            AnyDht::Header => Header::new().get_prefix(),
        }
    }
    fn hash_name(self) -> &'static str {
        "AnyDhtHash"
    }
}

impl HashTypeAsync for AnyDht {}

// FIXME: REMOVE [ B-02112 ]
impl Default for AnyDht {
    fn default() -> Self {
        // HACK: SO WRONG
        AnyDht::Header
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
enum AnyDhtSerial {
    /// The hash of an Entry of EntryType::Agent
    Header(Header),
    /// The hash of any other EntryType
    Entry(Entry),
}

impl From<AnyDht> for AnyDhtSerial {
    fn from(t: AnyDht) -> Self {
        match t {
            AnyDht::Header => AnyDhtSerial::Header(Header),
            AnyDht::Entry => AnyDhtSerial::Entry(Entry),
        }
    }
}

impl From<AnyDhtSerial> for AnyDht {
    fn from(t: AnyDhtSerial) -> Self {
        match t {
            AnyDhtSerial::Header(_) => AnyDht::Header,
            AnyDhtSerial::Entry(_) => AnyDht::Entry,
        }
    }
}
