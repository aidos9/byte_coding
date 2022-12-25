pub trait Encodable
where
    Self: Sized,
{
    fn encoded(&self) -> Vec<u8> {
        let mut buf = Vec::new();

        self.encode_to_buf(&mut buf);

        return buf;
    }

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

impl Encodable for i64 {
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

#[cfg(test)]
mod tests {
    use super::*;

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
