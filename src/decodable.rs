/// Provide methods to decode objects from a vector of bytes.
///
/// # Usage
/// The `decoded` method should be used when decoding a slice into an object, alternatively if decoding
/// multiple objects at a time, the `decode_from_buf` method can be used. Implementations have been
/// provided for some common types but for custom types or other types the trait can be implemented
/// using the below instructions.
///
/// ## Example
/// ```
/// use byte_coding::Decodable;
///
/// let data = vec![1, 4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't'];
/// let decoded: Option<String> = Decodable::decode(&data).unwrap();
///
/// // Call the encoded method to convert to a buffer
/// assert_eq!(Some("test".to_string()), decoded);
/// ```
///
/// # Implementing
/// When implementing this trait, you must implement the `decode_from_buf` method. This method should
/// consume as much of the input buffer as necessary to decode the object and return any left over in
/// the tuple as a result with the decoded object. This is to allow the chaining of multiple decodes.
/// If an error occurs or the input data is poorly formatted, you should return None to indicate to the
/// caller that an error occurred.
///
/// ## Example
/// The below example is an implementation of the trait for an example struct.
/// ```
/// use byte_coding::Decodable;
///
/// struct Example {
///     f1: String,
///     f2: u16
/// }
///
/// impl Decodable for Example {
///     fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
///         let (f1, buffer) = Decodable::decode_from_buf(buffer)?;
///         let (f2, buffer) = Decodable::decode_from_buf(buffer)?;
///
///         return Some((Example {f1, f2}, buffer));
///     }
/// }
/// ```
///
/// You can then encode the object like any other type
/// ```
/// # use byte_coding::Decodable;
/// # #[derive(Debug, PartialEq)]
/// # struct Example {
/// #     f1: String,
/// #     f2: u16
/// # }
/// #
/// # impl Decodable for Example {
/// #     fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
/// #         let (f1, buffer) = Decodable::decode_from_buf(buffer)?;
/// #         let (f2, buffer) = Decodable::decode_from_buf(buffer)?;
/// #
/// #         return Some((Example {f1, f2}, buffer));
/// #     }
/// # }
/// #
/// let data = vec![7, 0, 0, 0, 0, 0, 0, 0, b'e', b'x', b'a', b'm', b'p', b'l', b'e', 255, 255];
/// let decoded: Example = Decodable::decode(&data).unwrap();
///
/// assert_eq!(Example {
///     f1: "example".to_string(),
///     f2: 65535
/// }, decoded);
/// ```
pub trait Decodable
where
    Self: Sized,
{
    /// Decodes a slice of bytes into the object implemented on. If the decode fails, a `None`
    /// value is returned instead.
    ///
    /// ### Example
    /// ```
    /// use byte_coding::Decodable;
    ///
    /// let src = vec![255, 255];
    /// let decoded = u16::decode(&src).unwrap();
    /// assert_eq!(65535, decoded);
    /// ```
    fn decode(bytes: &[u8]) -> Option<Self> {
        return Self::decode_from_buf(bytes).map(|(v, _)| v);
    }

    /// Decodes a slice of bytes into the object implemented on, returns a slice of the input
    /// buffer which contains only unprocessed bytes. If the decode fails, a `None` value is
    /// returned instead.
    ///
    /// ### Example
    /// ```
    /// use byte_coding::Decodable;
    ///
    /// let src = vec![255, 255, 0, 0, 0];
    /// let (decoded, buffer) = u16::decode_from_buf(&src).unwrap();
    /// assert_eq!(65535, decoded);
    /// assert_eq!(buffer, &[0, 0, 0]);
    /// ```
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])>;
}

impl Decodable for u8 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        if buffer.len() < 1 {
            return None;
        }

        return Some((Self::from_le_bytes([buffer[0]]), &buffer[1..]));
    }
}

impl Decodable for u16 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        if buffer.len() < 2 {
            return None;
        }

        let mut bytes_array = [0u8; 2];

        bytes_array.copy_from_slice(&buffer[0..2]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[2..]));
    }
}

impl Decodable for u32 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        if buffer.len() < 4 {
            return None;
        }

        let mut bytes_array = [0u8; 4];

        bytes_array.copy_from_slice(&buffer[0..4]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[4..]));
    }
}

impl Decodable for u64 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        if buffer.len() < 8 {
            return None;
        }

        let mut bytes_array = [0u8; 8];

        bytes_array.copy_from_slice(&buffer[0..8]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[8..]));
    }
}

impl Decodable for u128 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        if buffer.len() < 16 {
            return None;
        }

        let mut bytes_array = [0u8; 16];

        bytes_array.copy_from_slice(&buffer[0..16]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[16..]));
    }
}

impl Decodable for i8 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        const BYTES: usize = 1;

        if buffer.len() < BYTES {
            return None;
        }

        let mut bytes_array = [0u8; BYTES];

        bytes_array.copy_from_slice(&buffer[0..BYTES]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[BYTES..]));
    }
}

impl Decodable for i16 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        const BYTES: usize = 2;

        if buffer.len() < BYTES {
            return None;
        }

        let mut bytes_array = [0u8; BYTES];

        bytes_array.copy_from_slice(&buffer[0..BYTES]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[BYTES..]));
    }
}

impl Decodable for i32 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        const BYTES: usize = 4;

        if buffer.len() < BYTES {
            return None;
        }

        let mut bytes_array = [0u8; BYTES];

        bytes_array.copy_from_slice(&buffer[0..BYTES]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[BYTES..]));
    }
}

impl Decodable for i64 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        const BYTES: usize = 8;

        if buffer.len() < BYTES {
            return None;
        }

        let mut bytes_array = [0u8; BYTES];

        bytes_array.copy_from_slice(&buffer[0..BYTES]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[BYTES..]));
    }
}

impl Decodable for i128 {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        const BYTES: usize = 16;

        if buffer.len() < BYTES {
            return None;
        }

        let mut bytes_array = [0u8; BYTES];

        bytes_array.copy_from_slice(&buffer[0..BYTES]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[BYTES..]));
    }
}

impl Decodable for usize {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        if buffer.len() < 8 {
            return None;
        }

        let mut bytes_array = [0u8; 8];

        bytes_array.copy_from_slice(&buffer[0..8]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[8..]));
    }
}

impl Decodable for isize {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        if buffer.len() < 8 {
            return None;
        }

        let mut bytes_array = [0u8; 8];

        bytes_array.copy_from_slice(&buffer[0..8]);

        return Some((Self::from_le_bytes(bytes_array), &buffer[8..]));
    }
}

impl<T: Decodable> Decodable for Box<T> {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        return T::decode_from_buf(buffer).map(|(v, a)| (Box::new(v), a));
    }
}

impl<T: Decodable> Decodable for Vec<T> {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        let (len, mut buffer) = usize::decode_from_buf(buffer)?;

        let mut vec = Vec::with_capacity(len);

        for _ in 0..len {
            let res = T::decode_from_buf(buffer)?;

            vec.push(res.0);
            buffer = res.1;
        }

        return Some((vec, buffer));
    }
}

impl<T: Decodable, const N: usize> Decodable for [T; N] {
    fn decode_from_buf(mut buffer: &[u8]) -> Option<(Self, &[u8])> {
        let mut vec = Vec::with_capacity(N);

        for _ in 0..N {
            let res = T::decode_from_buf(buffer)?;

            vec.push(res.0);
            buffer = res.1;
        }

        return Some((vec.try_into().ok()?, buffer));
    }
}

impl Decodable for [bool; 8] {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        if buffer.len() < 1 {
            return None;
        }

        let mut res = [false; 8];

        for i in 0..8 {
            res[7 - i] = (buffer[0] & (1 << i)) != 0;
        }

        return Some((res, &buffer[1..]));
    }
}

impl<T: Decodable> Decodable for Option<T> {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        let (present, buffer) = u8::decode_from_buf(buffer)?;

        if present == 0 {
            return Some((None, buffer));
        } else {
            return T::decode_from_buf(buffer).map(|(s, p)| (Some(s), p));
        }
    }
}

impl Decodable for String {
    fn decode_from_buf(buffer: &[u8]) -> Option<(Self, &[u8])> {
        let (len, buffer) = u64::decode_from_buf(buffer)?;

        return Some((
            String::from_utf8(buffer[0..len as usize].to_vec()).ok()?,
            &buffer[len as usize..],
        ));
    }
}

#[cfg(test)]
mod tests {
    use crate::Encodable;

    use super::*;

    #[test]
    fn test_u8() {
        let b = u8::MAX - 1;
        let encoded = b.encoded();
        let res = Decodable::decode(&encoded).unwrap();

        assert_eq!(b, res);
    }

    #[test]
    fn test_u16() {
        let b = u16::MAX - 1;
        let encoded = b.encoded();
        let res = Decodable::decode(&encoded).unwrap();

        assert_eq!(b, res);
    }

    #[test]
    fn test_u32() {
        let b = u32::MAX - 1;
        let encoded = b.encoded();
        let res = Decodable::decode(&encoded).unwrap();

        assert_eq!(b, res);
    }

    #[test]
    fn test_u64() {
        let b = u64::MAX - 1;
        let encoded = b.encoded();
        let res = Decodable::decode(&encoded).unwrap();

        assert_eq!(b, res);
    }

    #[test]
    fn test_string() {
        let b = "test";
        let encoded = b.encoded();
        let res: String = Decodable::decode(&encoded).unwrap();

        assert_eq!(b, res);
    }

    #[test]
    fn test_option_string_1() {
        let b = Some("test".to_string());
        let encoded = b.encoded();
        let res: Option<String> = Decodable::decode(&encoded).unwrap();

        assert_eq!(b, res);
    }

    #[test]
    fn test_option_string_2() {
        let b = None;
        let encoded = b.encoded();
        let res: Option<String> = Decodable::decode(&encoded).unwrap();

        assert_eq!(b, res);
    }

    #[test]
    fn test_bool_array() {
        let b = [true, false, true, false, true, false, true, false];
        let encoded = b.encoded();
        let res: [bool; 8] = Decodable::decode(&encoded).unwrap();

        assert_eq!(b, res);
    }

    #[test]
    fn test_vec() {
        let b = vec![1u64, 2, 3, 4];
        let encoded = b.encoded();
        let res: Vec<u64> = Decodable::decode(&encoded).unwrap();

        assert_eq!(b, res);
    }

    #[test]
    fn test_array() {
        let b = ["abc", "bc", "c", "defghijklmnopq"];
        let encoded = b.encoded();
        let res: [String; 4] = Decodable::decode(&encoded).unwrap();

        assert_eq!(b.map(|v| v.to_string()), res);
    }
}
