/// Represents a decoded bencode value with its original raw bytes
#[derive(Debug, Clone)]
pub struct BencodeValue {
    /// The decoded value
    pub kind: BencodeKind,
    /// The original raw bytes from the torrent file
    raw_bytes: Vec<u8>,
}

/// The actual decoded bencode type
#[derive(Debug, Clone)]
pub enum BencodeKind {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<BencodeValue>),
    Dictionary(Vec<(String, BencodeValue)>),
}

impl BencodeValue {
    /// Create a new BencodeValue with raw bytes
    pub fn new(kind: BencodeKind, raw_bytes: Vec<u8>) -> Self {
        BencodeValue { kind, raw_bytes }
    }

    /// Get the raw bytes of this value
    pub fn raw_bytes(&self) -> &[u8] {
        &self.raw_bytes
    }

    /// Get the string representation if this is a String variant
    pub fn as_string(&self) -> Option<&str> {
        match &self.kind {
            BencodeKind::String(bytes) => std::str::from_utf8(bytes).ok(),
            _ => None,
        }
    }

    /// Get the raw bytes if this is a String variant (for binary data)
    pub fn as_bytes(&self) -> Option<&[u8]> {
        match &self.kind {
            BencodeKind::String(bytes) => Some(bytes),
            _ => None,
        }
    }

    /// Get the integer value if this is an Integer variant
    pub fn as_integer(&self) -> Option<i64> {
        match &self.kind {
            BencodeKind::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Get a value from a dictionary by key
    pub fn get(&self, key: &str) -> Option<&BencodeValue> {
        match &self.kind {
            BencodeKind::Dictionary(entries) => {
                entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }
}
