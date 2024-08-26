use udigest::encoding::*;

use common::VecBuf;

mod common;

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

#[test]
fn encode_integers() {
    fn encoding(value: impl udigest::Digestable) -> Vec<u8> {
        let mut buf = VecBuf(vec![]);
        let encoder = EncodeValue::new(&mut buf);
        value.unambiguously_encode(encoder);
        buf.0
    }
    fn expect<T: udigest::Digestable + std::fmt::Debug>(value: T, expected: &[u8]) {
        let actual = hex::encode(encoding(&value));

        let expected_len = u32::try_from(expected.len()).unwrap().to_be_bytes();
        let expected = concat_bytes_into_vec!(expected, expected_len, [LEN_32, LEAF]);
        let expected = hex::encode(expected);
        assert_eq!(actual, expected, "encoding of {value:?}");
    }
    fn expect_eq<
        A: udigest::Digestable + std::fmt::Debug,
        B: udigest::Digestable + std::fmt::Debug,
    >(
        lhs: A,
        rhs: B,
    ) {
        let lhs = hex::encode(encoding(lhs));
        let rhs = hex::encode(encoding(rhs));
        assert_eq!(lhs, rhs, "{lhs:?} != {rhs:?}");
    }

    expect(0_u16, &[]);
    expect(1_u16, &[1]);
    expect(255_u16, &[255]);
    expect(256_u16, &[1, 0]);

    expect_eq(1_u16, 1_usize);
    expect_eq(1000_u16, 1000_usize);
    expect_eq(1_000_000_usize, 1_000_000_u64);

    expect(0_i16, &[]);
    expect(1_i16, &[1, 1]);
    expect(255_i16, &[1, 255]);
    expect(256_i16, &[1, 1, 0]);

    expect(-1i16, &[0, 1]);
    expect(-255_i16, &[0, 255]);
    expect(-256_i16, &[0, 1, 0]);

    expect_eq(1_i16, 1_isize);
    expect_eq(1000_i16, 1000_isize);
    expect_eq(1_000_000_isize, 1_000_000_i64);
}
