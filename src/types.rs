use heapless::String;

pub type Key = String<32>;    // Up to 32-byte key
pub type Value = String<128>; // Up to 128-byte value

pub const MAX_KEY_LEN: usize = 32;
pub const MAX_VALUE_LEN: usize = 128;
pub const RECORD_SIZE: usize = MAX_KEY_LEN + MAX_VALUE_LEN;

#[derive(Clone)]
pub struct Record {
    pub key: Key,
    pub value: Value,
}

impl Record {
    pub fn to_bytes(&self) -> [u8; RECORD_SIZE] {
        let mut buffer = [0u8; RECORD_SIZE];
        let key_bytes = self.key.as_bytes();
        let value_bytes = self.value.as_bytes();

        buffer[..key_bytes.len()].copy_from_slice(key_bytes);
        buffer[MAX_KEY_LEN..MAX_KEY_LEN + value_bytes.len()].copy_from_slice(value_bytes);

        buffer
    }

    pub fn from_bytes(bytes: &[u8]) -> Option<Self> {
        if bytes.len() != RECORD_SIZE {
            return None;
        }

        let key_slice = &bytes[..MAX_KEY_LEN];
        let value_slice = &bytes[MAX_KEY_LEN..];

        let key_str = core::str::from_utf8(key_slice).ok()?.trim_end_matches('\0');
        let value_str = core::str::from_utf8(value_slice).ok()?.trim_end_matches('\0');

        let mut key = Key::new();
        let mut value = Value::new();

        key.push_str(key_str).ok()?;
        value.push_str(value_str).ok()?;

        Some(Self { key, value })
    }
}