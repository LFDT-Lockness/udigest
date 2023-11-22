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

    let expected = concat_bytes_into_vec!(
        // "1234" 4_u32 LEN_32 LEAF
        b"1234",
        4_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // "1" 1_u32 LEN_32 LEAF
        b"1",
        1_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // "2" 1_u32 LEN_32 LEAF
        b"2",
        1_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // 2_u32 LEN_32 LIST
        2_u32.to_be_bytes(),
        [LEN_32, LIST],
        // "abc" 3_u32 LEN_32 LEAF
        b"abc",
        3_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // 3_u32 LEN_32 LIST
        3_u32.to_be_bytes(),
        [LEN_32, LIST]
    );

    assert_eq!(buffer.0, expected);
}

#[test]
fn encode_with_tag() {
    // Encode "123" with tag "SOME_TAG"
    let mut buffer = VecBuf(vec![]);
    EncodeLeaf::new(&mut buffer)
        .with_tag(b"SOME_TAG")
        .chain(b"123");

    let expected = concat_bytes_into_vec!(
        b"123",
        3_u32.to_be_bytes(),
        [LEN_32],
        b"SOME_TAG",
        8_u32.to_be_bytes(),
        [LEN_32, LEAF_CTX]
    );
    assert_eq!(buffer.0, expected);

    // Encode `[]` with tag "SOME_TAG"
    let mut buffer = VecBuf(vec![]);
    EncodeList::new(&mut buffer).with_tag(b"SOME_TAG");

    let expected = concat_bytes_into_vec!(
        0_u32.to_be_bytes(),
        [LEN_32],
        b"SOME_TAG",
        8_u32.to_be_bytes(),
        [LEN_32, LIST_CTX]
    );
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
        [LEN_32, LEAF],
        // "Alice" 5_u32 LEN_32 LEAF
        "Alice",
        5_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // "skills" 6_u32 LEN_32 LEAF
        "skills",
        6_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // "math" 4_u32 LEN_32 LEAF
        "math",
        4_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // "crypto" 6_u32 LEN_32 LEAF
        "crypto",
        6_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // 2_u32 LEN_32 LIST
        2_u32.to_be_bytes(),
        [LEN_32, LIST],
        // "job_title" 9_u32 LEN_32 LEAF
        "job_title",
        9_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // "cryptographer" 13_u32 LEN_32 LEAF
        "cryptographer",
        13_u32.to_be_bytes(),
        [LEN_32, LEAF],
        // 6_u32 LEN_32 LIST
        6_u32.to_be_bytes(),
        [LEN_32, LIST],
    );

    assert_eq!(buffer.0, expected);
}

#[test]
fn encode_biglen() {
    assert_eq!(
        usize::BITS,
        64,
        "this test needs to be executed on 64-bits platform"
    );

    let len = 0x0100000000_usize;

    let mut buf = VecBuf(vec![]);
    encode_len(&mut buf, len);

    assert_eq!(buf.0, [1, 0, 0, 0, 0, 5, BIGLEN]);
}
