#[cfg(feature = "std")]
use std::collections::HashMap;

#[cfg(not(feature = "std"))]
use alloc::boxed::Box;
#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;

/// Provide methods to encode objects into a vector of bytes.
///
/// # Usage
/// The `encoded` method should be used when encoding an entire object, alternatively if encoding
/// multiple objects at a time, the `encode_to_buf` method can be used. Implementations have been
/// provided for some common types but for custom types or other types the trait can be implemented
/// using the below instructions.
///
/// ## Example
/// ```
/// use byte_coding::Encodable;
///
/// let data = Some("test");
///
/// // Call the encoded method to convert to a buffer
/// assert_eq!(data.encoded(), vec![1, 4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't']);
///
/// // Alternatively uuse the encode_to_buf method to append to an existing buffer
/// let mut buf = Vec::new();
/// data.encode_to_buf(&mut buf);
/// assert_eq!(buf, vec![1, 4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't']);
/// ```
///
/// # Implementing
/// When implementing this trait, simply implement the `encode_to_buf` method. Any data should be
/// appended to the buffer provided as an argument. Where possible utilise the existing methods
/// provided for other types.
///
/// ## Example
/// The below example is an implementation of the trait for an example struct.
/// ```
/// use byte_coding::Encodable;
///
/// struct Example {
///     f1: String,
///     f2: u16
/// }
///
/// impl Encodable for Example {
///     fn encode_to_buf(&self, buf: &mut Vec<u8>) {
///         self.f1.encode_to_buf(buf);
///         self.f2.encode_to_buf(buf);
///     }
/// }
/// ```
///
/// You can then encode the object like any other type
/// ```
/// # use byte_coding::Encodable;
/// #
/// # struct Example {
/// #     f1: String,
/// #     f2: u16
/// # }
/// #
/// # impl Encodable for Example {
/// #     fn encode_to_buf(&self, buf: &mut Vec<u8>) {
/// #         self.f1.encode_to_buf(buf);
/// #         self.f2.encode_to_buf(buf);
/// #     }
/// # }
/// #
/// let example = Example {
///     f1: "example".to_string(),
///     f2: 65535
/// };
///
/// assert_eq!(example.encoded(), vec![7, 0, 0, 0, 0, 0, 0, 0, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 255, 255]);
/// ```

pub trait Encodable
where
    Self: Sized,
{
    /// Returns a vector of bytes representing this object.
    ///
    /// ### Example
    /// ```
    /// use byte_coding::Encodable;
    ///
    /// let encoded = 65535u16.encoded();
    /// assert_eq!(vec![255, 255], encoded);
    /// ```
    fn encoded(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        self.encode_to_buf(&mut buf);

        return buf;
    }

    /// Append the bytes to the provided buffer which represent this object.
    ///
    /// ### Example
    /// ```
    /// use byte_coding::Encodable;
    ///
    /// let mut encoded = vec![0, 0];
    ///
    /// 65535u16.encode_to_buf(&mut encoded);
    ///
    /// assert_eq!(vec![0, 0, 255, 255], encoded);
    /// ```
    fn encode_to_buf(&self, buf: &mut Vec<u8>);
}

impl<T: Encodable> Encodable for Option<T> {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        match self {
            Some(s) => {
                1u8.encode_to_buf(buf);
                s.encode_to_buf(buf);
            }
            None => 0u8.encode_to_buf(buf),
        }
    }
}

impl Encodable for &str {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        self.len().encode_to_buf(buf);
        buf.extend_from_slice(self.as_bytes());
    }
}

impl Encodable for String {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        self.as_str().encode_to_buf(buf);
    }
}

impl Encodable for u8 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encodable for u16 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}
impl Encodable for u32 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encodable for u64 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encodable for u128 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encodable for i8 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encodable for i16 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encodable for i32 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encodable for i64 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encodable for i128 {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(&self.to_le_bytes());
    }
}

impl Encodable for usize {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        (*self as u64).encode_to_buf(buf);
    }
}

impl Encodable for isize {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        (*self as i64).encode_to_buf(buf);
    }
}

impl<T: Encodable> Encodable for Box<T> {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        self.as_ref().encode_to_buf(buf);
    }
}

impl<T: Encodable> Encodable for Vec<T> {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        self.as_slice().encode_to_buf(buf);
    }
}

impl<T: Encodable> Encodable for &[T] {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        self.len().encode_to_buf(buf);

        for item in self.iter() {
            item.encode_to_buf(buf);
        }
    }
}

impl<T: Encodable, const N: usize> Encodable for [T; N] {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        for item in self.iter() {
            item.encode_to_buf(buf);
        }
    }
}

impl Encodable for [bool; 8] {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        let mut byte: u8 = 0b0000_0000;

        for b in self {
            byte <<= 1;

            if *b {
                byte |= 0b0000_0001;
            }
        }

        byte.encode_to_buf(buf);
    }
}

impl<T: Encodable> Encodable for &T {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        (*self).encode_to_buf(buf);
    }
}

#[cfg(feature = "std")]
impl<K: Encodable, V: Encodable> Encodable for HashMap<K, V> {
    fn encode_to_buf(&self, buf: &mut Vec<u8>) {
        self.len().encode_to_buf(buf);

        for (k, v) in self {
            k.encode_to_buf(buf);
            v.encode_to_buf(buf);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(not(feature = "std"))]
    use alloc::vec;

    #[test]
    fn test_encoding_bool() {
        let bools = [true, true, false, false, true, true, false, false];
        assert_eq!(bools.encoded(), vec![0b11001100]);
    }

    #[test]
    fn test_encoding_bool_2() {
        let bools = [true, false, true, false, true, false, true, false];
        assert_eq!(bools.encoded(), vec![0b10101010]);
    }

    #[test]
    fn test_encoding_u64() {
        assert_eq!(52u64.encoded(), vec![52, 0, 0, 0, 0, 0, 0, 0]);
    }

    #[test]
    fn test_encoding_i64() {
        assert_eq!(
            (-52i64).encoded(),
            vec![204, 255, 255, 255, 255, 255, 255, 255]
        );
    }

    #[test]
    fn test_encoding_vec_u64() {
        assert_eq!(
            vec![123u64, 1024u64, 18u64].encoded(),
            vec![
                3, 0, 0, 0, 0, 0, 0, 0, 123, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 18, 0, 0,
                0, 0, 0, 0, 0
            ]
        );
    }

    #[test]
    fn test_encoding_arr_u64() {
        assert_eq!(
            [123u64, 1024u64, 18u64].encoded(),
            vec![123, 0, 0, 0, 0, 0, 0, 0, 0, 4, 0, 0, 0, 0, 0, 0, 18, 0, 0, 0, 0, 0, 0, 0]
        );
    }

    #[test]
    fn test_encoding_str() {
        assert_eq!(
            "test".encoded(),
            vec![4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't']
        );
    }
}
