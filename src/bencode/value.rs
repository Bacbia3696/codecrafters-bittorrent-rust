/// Represents a decoded bencode value
#[derive(Debug, Clone)]
pub enum BencodeValue {
    Integer(i64),
    String(Vec<u8>),
    List(Vec<BencodeValue>),
    Dictionary(Vec<(String, BencodeValue)>),
}

impl BencodeValue {
    /// Get the string representation if this is a String variant
    pub fn as_string(&self) -> Option<&str> {
        match self {
            BencodeValue::String(bytes) => std::str::from_utf8(bytes).ok(),
            _ => None,
        }
    }

    /// Get the integer value if this is an Integer variant
    pub fn as_integer(&self) -> Option<i64> {
        match self {
            BencodeValue::Integer(n) => Some(*n),
            _ => None,
        }
    }

    /// Get a value from a dictionary by key
    pub fn get(&self, key: &str) -> Option<&BencodeValue> {
        match self {
            BencodeValue::Dictionary(entries) => {
                entries.iter().find(|(k, _)| k == key).map(|(_, v)| v)
            }
            _ => None,
        }
    }
}
