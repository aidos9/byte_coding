use crate::{Decodable, Encodable};

/// This object is a simple frontend that is useful when encoding or decoding multiple
/// objects where a new struct is not desired. To use this object enable the 'coder'
/// feature. This feature is enabled by default.
///
/// ## Example
/// ```
/// use byte_coding::Coder;
///
/// let mut coder = Coder::new();
///
/// let src_str = "object";
///
/// coder.encode(&src_str);
///
/// let decoded_str: String = coder.decode_next_object().unwrap();
/// assert_eq!(src_str, decoded_str);
/// ```
#[derive(Clone, Debug, PartialEq, Hash)]
pub struct Coder {
    buffer: Vec<u8>,
    decode_index: usize,
}

impl Coder {
    /// Creates a new coder with an empty buffer.
    pub fn new() -> Self {
        return Self::default();
    }

    /// Creates a new coder using the specified buffer.
    pub fn with_buffer(buffer: Vec<u8>) -> Self {
        return Self {
            buffer,
            decode_index: 0,
        };
    }

    /// This resets the coder back to the start
    pub fn reset_decode_index(&mut self) {
        self.decode_index = 0;
    }

    /// Returns a reference to the underlying buffer
    pub fn buffer(&self) -> &Vec<u8> {
        return &self.buffer;
    }

    /// Encode an object to the internal buffer.
    pub fn encode<T: Encodable>(&mut self, object: &T) {
        object.encode_to_buf(&mut self.buffer);
    }

    /// Encode an object to the internal buffer.
    pub fn encode_object<T: Encodable>(&mut self, object: T) {
        object.encode_to_buf(&mut self.buffer);
    }

    /// Attempts to decode an object from the buffer continuing from the previously decoded object.
    pub fn decode_next_object<T: Decodable>(&mut self) -> Option<T> {
        let (res, b) = T::decode_from_buf(&self.buffer[self.decode_index..])?;

        self.decode_index =
            unsafe { b.as_ptr_range().start.offset_from(self.buffer.as_ptr()) } as usize;

        return Some(res);
    }
}

impl Default for Coder {
    fn default() -> Self {
        return Self {
            buffer: Vec::new(),
            decode_index: 0,
        };
    }
}

impl From<&[u8]> for Coder {
    fn from(value: &[u8]) -> Self {
        return Self::with_buffer(value.to_vec());
    }
}

impl From<Vec<u8>> for Coder {
    fn from(value: Vec<u8>) -> Self {
        return Self::with_buffer(value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_next_object() {
        let mut coder = Coder::from(vec![0xff, 0xff, 0xee, 0xee, 0xdd, 0xdd]);

        let f: u16 = coder.decode_next_object().unwrap();
        assert_eq!(f, 0xffff);

        let e: u16 = coder.decode_next_object().unwrap();
        assert_eq!(e, 0xeeee);

        let d: u16 = coder.decode_next_object().unwrap();
        assert_eq!(d, 0xdddd);
    }

    #[test]
    fn test_encode() {
        let mut coder = Coder::new();

        coder.encode_object(0xffffu16);
        coder.encode_object(0xeeeeu16);
        coder.encode_object(0xddddu16);

        let f: u16 = coder.decode_next_object().unwrap();
        assert_eq!(f, 0xffff);

        let e: u16 = coder.decode_next_object().unwrap();
        assert_eq!(e, 0xeeee);

        let d: u16 = coder.decode_next_object().unwrap();
        assert_eq!(d, 0xdddd);
    }

    #[test]
    fn test_encode_2() {
        let mut coder = Coder::new();

        coder.encode_object("test");
        coder.encode_object("test2");
        coder.encode_object("test3");

        let f: String = coder.decode_next_object().unwrap();
        assert_eq!(f, "test".to_string());

        let e: String = coder.decode_next_object().unwrap();
        assert_eq!(e, "test2".to_string());

        let d: String = coder.decode_next_object().unwrap();
        assert_eq!(d, "test3".to_string());
    }
}
