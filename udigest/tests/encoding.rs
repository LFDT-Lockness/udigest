use udigest::encoding::*;

/// A buffer based on `Vec<u8>`. Writing to the buffer
/// appends data to the vector
pub struct VecBuf(Vec<u8>);

impl udigest::encoding::Buffer for VecBuf {
    fn write(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes)
    }
}

macro_rules! concat_bytes_into_vec {
    ($($bytes:expr),*$(,)?) => {{
        let mut concated = vec![];
        $(
            let bytes = $bytes;
            let bytes: &[u8] = bytes.as_ref();
            concated.extend_from_slice(bytes);
        )*
        concated
    }};
}

#[test]
fn simple_encoding() {
    // Encode:
    // ["1234", ["1", "2"], "abc"]
    let mut buffer = VecBuf(vec![]);

    let mut list = EncodeList::new(&mut buffer);
    list.add_leaf().chain(b"1234");

    let mut sublist = list.add_list();
    sublist.add_leaf().chain(b"1");
    sublist.add_leaf().chain(b"2");
    sublist.finish();

    list.add_leaf().chain(b"abc");
    list.finish();

    let mut expected = vec![];

    // "1234" 4_u32 LEN_32 LEAF
    expected.extend_from_slice(b"1234");
    expected.extend_from_slice(&4_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32, LEAF]);
    // "1" 1_u32 LEN_32 LEAF
    expected.extend_from_slice(b"1");
    expected.extend_from_slice(&1_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32, LEAF]);
    // "2" 1_u32 LEN_32 LEAF
    expected.extend_from_slice(b"2");
    expected.extend_from_slice(&1_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32, LEAF]);
    // 2_u32 LEN_32 LIST
    expected.extend_from_slice(&2_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32, LIST]);
    // "abc" 3_u32 LEN_32 LEAF
    expected.extend_from_slice(b"abc");
    expected.extend_from_slice(&3_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32, LEAF]);
    // 3_u32 LEN_32 LIST
    expected.extend_from_slice(&3_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32, LIST]);

    assert_eq!(buffer.0, expected);
}

#[test]
fn encode_with_tag() {
    // Encode "123" with tag "SOME_TAG"
    let mut buffer = VecBuf(vec![]);
    EncodeLeaf::new(&mut buffer)
        .with_tag(b"SOME_TAG")
        .chain(b"123");

    let mut expected = vec![];
    expected.extend_from_slice(b"123");
    expected.extend_from_slice(&3_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32]);
    expected.extend_from_slice(b"SOME_TAG");
    expected.extend_from_slice(&8_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32, LEAF_CTX]);

    assert_eq!(buffer.0, expected);

    // Encode `[]` with tag "SOME_TAG"
    let mut buffer = VecBuf(vec![]);
    EncodeList::new(&mut buffer).with_tag(b"SOME_TAG");

    let mut expected = vec![];
    expected.extend_from_slice(&0_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32]);
    expected.extend_from_slice(b"SOME_TAG");
    expected.extend_from_slice(&8_u32.to_be_bytes());
    expected.extend_from_slice(&[LEN_32, LIST_CTX]);

    assert_eq!(buffer.0, expected);
}

#[test]
fn encode_struct() {
    let mut buffer = VecBuf(vec![]);

    let mut s = EncodeStruct::new(&mut buffer);
    s.add_field("name").encode_leaf().chain("Alice");

    let mut skills = s.add_field("skills").encode_list();
    skills.add_leaf().chain("math");
    skills.add_leaf().chain("crypto");
    skills.finish();

    s.add_field("job_title")
        .encode_leaf()
        .chain("cryptographer");

    s.finish();

    let expected = concat_bytes_into_vec!(
        // "name" 4_u32 LEN_32 LEAF
        "name",
        4_u32.to_be_bytes(),
        &[LEN_32, LEAF],
        // "Alice" 5_u32 LEN_32 LEAF
        "Alice",
        5_u32.to_be_bytes(),
        &[LEN_32, LEAF],
        // "skills" 6_u32 LEN_32 LEAF
        "skills",
        6_u32.to_be_bytes(),
        &[LEN_32, LEAF],
        // "math" 4_u32 LEN_32 LEAF
        "math",
        4_u32.to_be_bytes(),
        &[LEN_32, LEAF],
        // "crypto" 6_u32 LEN_32 LEAF
        "crypto",
        6_u32.to_be_bytes(),
        &[LEN_32, LEAF],
        // 2_u32 LEN_32 LIST
        2_u32.to_be_bytes(),
        &[LEN_32, LIST],
        // "job_title" 9_u32 LEN_32 LEAF
        "job_title",
        9_u32.to_be_bytes(),
        &[LEN_32, LEAF],
        // "cryptographer" 13_u32 LEN_32 LEAF
        "cryptographer",
        13_u32.to_be_bytes(),
        &[LEN_32, LEAF],
        // 6_u32 LEN_32 LIST
        6_u32.to_be_bytes(),
        &[LEN_32, LIST],
    );

    assert_eq!(buffer.0, expected);
}
