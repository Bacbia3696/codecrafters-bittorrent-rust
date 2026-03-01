use super::value::{BencodeKind, BencodeValue};

/// Parser for bencoded data
pub struct BencodeParser<'a> {
    data: &'a [u8],
    pos: usize,
}

impl<'a> BencodeParser<'a> {
    pub fn new(data: &'a [u8]) -> Self {
        BencodeParser { data, pos: 0 }
    }

    fn peek(&self) -> Option<u8> {
        self.data.get(self.pos).copied()
    }

    fn advance(&mut self) -> Option<u8> {
        let byte = self.data.get(self.pos).copied();
        self.pos += 1;
        byte
    }

    /// Get current position
    fn pos(&self) -> usize {
        self.pos
    }

    /// Parse a bencode value, tracking raw bytes
    pub fn parse(&mut self) -> Result<BencodeValue, String> {
        let start = self.pos();
        let kind = match self.peek() {
            Some(b'i') => self.parse_integer()?,
            Some(b'l') => self.parse_list()?,
            Some(b'd') => self.parse_dictionary()?,
            Some(b) if b.is_ascii_digit() => self.parse_string()?,
            Some(b) => return Err(format!("Unexpected byte: {}", b)),
            None => return Err("Unexpected end of data".to_string()),
        };
        let end = self.pos();

        // Capture the raw bytes for this value
        let raw_bytes = self.data[start..end].to_vec();
        Ok(BencodeValue::new(kind, raw_bytes))
    }

    /// Parse an integer: i<number>e
    fn parse_integer(&mut self) -> Result<BencodeKind, String> {
        self.advance(); // consume 'i'

        let start = self.pos;
        while let Some(byte) = self.peek() {
            if byte == b'e' {
                break;
            }
            self.advance();
        }

        let end = self.pos;
        self.advance(); // consume 'e'

        let num_str = std::str::from_utf8(&self.data[start..end])
            .map_err(|_| "Invalid UTF-8 in integer")?;
        let num = num_str
            .parse::<i64>()
            .map_err(|_| format!("Invalid integer: {}", num_str))?;

        Ok(BencodeKind::Integer(num))
    }

    /// Parse a string: <length>:<data>
    fn parse_string(&mut self) -> Result<BencodeKind, String> {
        let colon_pos = self.data[self.pos..]
            .iter()
            .position(|&b| b == b':')
            .ok_or("Missing colon in string")?;

        let len_str = std::str::from_utf8(&self.data[self.pos..self.pos + colon_pos])
            .map_err(|_| "Invalid UTF-8 in string length")?;
        let length = len_str
            .parse::<usize>()
            .map_err(|_| format!("Invalid string length: {}", len_str))?;

        self.pos += colon_pos + 1; // skip past the colon

        let start = self.pos;
        self.pos += length;

        Ok(BencodeKind::String(self.data[start..self.pos].to_vec()))
    }

    /// Parse a list: l<elements>e
    fn parse_list(&mut self) -> Result<BencodeKind, String> {
        self.advance(); // consume 'l'

        let mut elements = Vec::new();
        while self.peek() != Some(b'e') {
            elements.push(self.parse()?);
        }

        self.advance(); // consume 'e'
        Ok(BencodeKind::List(elements))
    }

    /// Parse a dictionary: d<key1><value1>...e
    fn parse_dictionary(&mut self) -> Result<BencodeKind, String> {
        self.advance(); // consume 'd'

        let mut entries = Vec::new();
        while self.peek() != Some(b'e') {
            let key = self.parse()?;
            let key_str = key
                .as_string()
                .ok_or("Dictionary keys must be strings")?
                .to_string();

            let value = self.parse()?;
            entries.push((key_str, value));
        }

        self.advance(); // consume 'e'
        Ok(BencodeKind::Dictionary(entries))
    }
}

/// Decode bencoded bytes into a BencodeValue
pub fn decode_bencode(data: &[u8]) -> Result<BencodeValue, String> {
    let mut parser = BencodeParser::new(data);
    parser.parse()
}
