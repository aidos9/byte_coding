#[cfg(feature = "derive")]
mod derive_tests {
    use byte_coding::*;

    #[derive(Encodable, Decodable, Debug, PartialEq)]
    struct Example1(String);

    #[derive(Encodable, Decodable, Debug, PartialEq)]
    #[byte_coding(pre_enc_func = "change_example2", post_enc_func = "append_data_e2")]
    #[byte_coding(
        post_dec_func = "undo_change_example2",
        pre_dec_func = "undo_append_data_e2"
    )]
    struct Example2 {
        #[byte_coding(order_no = 1)]
        a: String,
        #[byte_coding(order_no = 0)]
        b: String,
    }

    #[derive(Encodable, Decodable, Debug, PartialEq)]
    #[repr(u32)]
    #[byte_coding(encoding_type = "u16")]
    enum Example3 {
        A1 = 1,
        #[byte_coding(value = 2)]
        A2,
        #[byte_coding(value = 3)]
        A3 {
            f1: u32,
            f2: u64,
        },
        #[byte_coding(value = 4)]
        A4(u32, u64),
    }

    #[derive(Encodable, Decodable, Debug, PartialEq, Clone)]
    #[byte_coding(pre_enc_func = "Self::make_f2_none")]
    struct Example4 {
        f1: String,
        f2: Option<String>,
    }

    #[derive(Encodable, Decodable, Debug, PartialEq, Clone)]
    struct Example5 {
        f1: String,
        #[byte_coding(ignore)]
        f2: Option<String>,
    }

    impl Example4 {
        fn make_f2_none(e4: &Example4) -> Example4 {
            return Example4 {
                f2: None,
                f1: e4.f1.clone(),
            };
        }
    }

    fn change_example2(e2: &Example2) -> Example2 {
        return Example2 {
            a: "cows".to_string(),
            b: e2.b.clone(),
        };
    }

    fn append_data_e2(buffer: &mut Vec<u8>) {
        buffer.push(b'h');
    }

    fn undo_change_example2(v: Example2, buffer: &[u8]) -> Option<(Example2, &[u8])> {
        return Some((
            Example2 {
                a: "cows_undone".to_string(),
                b: v.b.clone(),
            },
            buffer,
        ));
    }

    fn undo_append_data_e2(buffer: &[u8]) -> Option<&[u8]> {
        if buffer.len() == 0 || buffer[buffer.len() - 1] != b'h' {
            return None;
        }

        return Some(&buffer[..buffer.len() - 1]);
    }

    mod encoding {
        use super::*;

        #[test]
        fn test_example1_encoding() {
            let example1 = Example1("test".to_string());

            assert_eq!(
                example1.encoded(),
                vec![4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't']
            );
        }

        #[test]
        fn test_example2_encoding() {
            let example2 = Example2 {
                a: "test".to_string(),
                b: "dogs".to_string(),
            };

            assert_eq!(
                example2.encoded(),
                vec![
                    4, 0, 0, 0, 0, 0, 0, 0, b'd', b'o', b'g', b's', 4, 0, 0, 0, 0, 0, 0, 0, b'c',
                    b'o', b'w', b's', b'h'
                ]
            );
        }

        #[test]
        fn test_example3_encoding_1() {
            let value = Example3::A1;

            assert_eq!(value.encoded(), vec![1, 0]);
        }

        #[test]
        fn test_example3_encoding_2() {
            let value = Example3::A2;

            assert_eq!(value.encoded(), vec![2, 0]);
        }

        #[test]
        fn test_example3_encoding_3() {
            let value = Example3::A3 { f1: 100, f2: 200 };

            assert_eq!(
                value.encoded(),
                vec![3, 0, 100, 0, 0, 0, 200, 0, 0, 0, 0, 0, 0, 0]
            );
        }

        #[test]
        fn test_example3_encoding_4() {
            let value = Example3::A4(100, 200);

            assert_eq!(
                value.encoded(),
                vec![4, 0, 100, 0, 0, 0, 200, 0, 0, 0, 0, 0, 0, 0]
            );
        }

        #[test]
        fn test_example4_encoding() {
            let value = Example4 {
                f1: "f1".to_string(),
                f2: Some("field_2".to_string()),
            };

            assert_eq!(value.encoded(), vec![2, 0, 0, 0, 0, 0, 0, 0, b'f', b'1', 0]);
        }

        #[test]
        fn test_example5_encoding() {
            let value = Example5 {
                f1: "f1".to_string(),
                f2: Some("field_2".to_string()),
            };

            assert_eq!(value.encoded(), vec![2, 0, 0, 0, 0, 0, 0, 0, b'f', b'1']);
        }
    }

    mod decoding {
        use super::*;

        #[test]
        fn test_example1_decoding() {
            let example1 = Example1("test".to_string());
            let encoded = example1.encoded();

            assert_eq!(
                encoded,
                vec![4, 0, 0, 0, 0, 0, 0, 0, b't', b'e', b's', b't']
            );

            let decoded = Decodable::decode(&encoded).unwrap();

            assert_eq!(example1, decoded);
        }

        #[test]
        fn test_example2_decoding() {
            let example2 = Example2 {
                a: "test".to_string(),
                b: "dogs".to_string(),
            };

            let encoded = example2.encoded();
            let decoded = Decodable::decode(&encoded).unwrap();

            assert_eq!(
                Example2 {
                    a: "cows_undone".to_string(),
                    b: "dogs".to_string(),
                },
                decoded
            );
        }

        #[test]
        fn test_example3_decoding_1() {
            let value = Example3::A1;

            let encoded = value.encoded();
            let decoded = Decodable::decode(&encoded).unwrap();

            assert_eq!(value, decoded);
        }

        #[test]
        fn test_example3_decoding_2() {
            let value = Example3::A2;

            let encoded = value.encoded();
            let decoded = Decodable::decode(&encoded).unwrap();

            assert_eq!(value, decoded);
        }

        #[test]
        fn test_example3_decoding_3() {
            let value = Example3::A3 { f1: 100, f2: 200 };

            let encoded = value.encoded();
            let decoded = Decodable::decode(&encoded).unwrap();

            assert_eq!(value, decoded);
        }

        #[test]
        fn test_example3_decoding_4() {
            let value = Example3::A4(100, 200);

            let encoded = value.encoded();
            let decoded = Decodable::decode(&encoded).unwrap();

            assert_eq!(value, decoded);
        }

        #[test]
        fn test_example5_decoding() {
            let value = Example5 {
                f1: "f1".to_string(),
                f2: Some("field_2".to_string()),
            };

            let encoded = value.encoded();
            let decoded: Example5 = Decodable::decode(&encoded).unwrap();

            assert_eq!(
                decoded,
                Example5 {
                    f1: "f1".to_string(),
                    f2: Default::default(),
                }
            );
        }
    }
}
