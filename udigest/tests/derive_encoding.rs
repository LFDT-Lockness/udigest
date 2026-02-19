//! In this test, we use proc macro with all possible attributes combinations to implement
//! `Digestable` trait, and then assert that encoding is as expected
//!
//! This test ensures that encoding stays compatible between versions of `udigest`. If
//! any of those tests break, it likely indicates a breaking change in the encoding.

mod common;

#[derive(udigest::Digestable)]
enum Shape {
    Circle {
        r: u16,
        color: Color,
    },
    #[udigest(rename = "Rectangular")]
    Rect {
        w: u16,
        h: u16,
        #[udigest(skip)]
        surface: u32,
        #[udigest(with = funny_color_encoding)]
        color: Color,
    },
    Square {
        #[udigest(rename = "width")]
        #[udigest(as = LittleEndian)]
        w: u16,
        #[udigest(as_bytes)]
        color: [u8; 3],
    },
    Generic {
        name: String,
        #[udigest(as_bytes = Wrapper::as_bytes)]
        wrapper: Wrapper,
        points: Vec<(u8, u8)>,
    },
}

#[derive(udigest::Digestable)]
#[udigest(tag = "regular color")]
struct Color {
    r: u16,
    g: u16,
    b: u16,
}

fn funny_color_encoding(color: &Color, encoder: udigest::encoding::EncodeValue) {
    let mut list = encoder.encode_list().with_tag(b"funny color");
    list.add_leaf().chain(color.b.to_be_bytes());
    list.add_leaf().chain(color.g.to_be_bytes());
    list.add_leaf().chain(color.r.to_be_bytes());
    list.finish();
}

struct LittleEndian;

impl udigest::as_::DigestAs<u16> for LittleEndian {
    fn digest_as(value: &u16, encoder: udigest::encoding::EncodeValue) {
        encoder.encode_leaf_value(value.to_le_bytes());
    }
}

struct Wrapper(Vec<u8>);

impl Wrapper {
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[test]
fn shape_circle() {
    // This test ensures that encoding stays compatible between versions of `udigest`. If
    // any of those tests break, it likely indicates a breaking change in the encoding.

    let circle = Shape::Circle {
        r: 10,
        color: Color {
            r: 12,
            g: 13,
            b: 14,
        },
    };
    let actual = common::encode_to_vec(&circle);

    let mut expected = common::VecBuf::default();
    let mut s = udigest::encoding::EncodeValue::new(&mut expected)
        .encode_enum()
        .with_variant("Circle");
    s.add_field("r").encode(&10_u8);
    let mut color = s
        .add_field("color")
        .encode_struct()
        .with_tag(b"regular color");
    color.add_field("r").encode(&12_u16);
    color.add_field("g").encode(&13_u16);
    color.add_field("b").encode(&14_u16);
    color.finish();
    s.finish();

    assert_eq!(hex::encode(actual), hex::encode(expected.0));
}

#[test]
fn shape_rect() {
    // This test ensures that encoding stays compatible between versions of `udigest`. If
    // any of those tests break, it likely indicates a breaking change in the encoding.

    let rect = Shape::Rect {
        w: 10,
        h: 11,
        surface: 10 * 11,
        color: Color {
            r: 100,
            g: 101,
            b: 102,
        },
    };
    let actual = common::encode_to_vec(&rect);

    let mut expected = common::VecBuf::default();
    let mut s = udigest::encoding::EncodeValue::new(&mut expected)
        .encode_enum()
        .with_variant("Rectangular");
    s.add_field("w").encode(&10_u16);
    s.add_field("h").encode(&11_u16);
    let mut color = s.add_field("color").encode_list().with_tag(b"funny color");
    color.add_leaf().chain(102_u16.to_be_bytes());
    color.add_leaf().chain(101_u16.to_be_bytes());
    color.add_leaf().chain(100_u16.to_be_bytes());
    color.finish();
    s.finish();

    assert_eq!(hex::encode(actual), hex::encode(expected.0));
}

#[test]
fn shape_square() {
    // This test ensures that encoding stays compatible between versions of `udigest`. If
    // any of those tests break, it likely indicates a breaking change in the encoding.

    let square = Shape::Square {
        w: 1001,
        color: [1, 2, 3],
    };
    let actual = common::encode_to_vec(&square);

    let mut expected = common::VecBuf::default();
    let mut s = udigest::encoding::EncodeValue::new(&mut expected)
        .encode_enum()
        .with_variant("Square");
    s.add_field("width")
        .encode_leaf_value(1001_u16.to_le_bytes());
    s.add_field("color").encode_leaf_value([1, 2, 3]);
    s.finish();

    assert_eq!(hex::encode(actual), hex::encode(expected.0));
}

#[test]
fn shape_generic() {
    // This test ensures that encoding stays compatible between versions of `udigest`. If
    // any of those tests break, it likely indicates a breaking change in the encoding.

    let shape = Shape::Generic {
        name: "triangle".to_owned(),
        wrapper: Wrapper(vec![10, 11, 100]),
        points: vec![(0, 0), (1, 0), (0, 1)],
    };
    let actual = common::encode_to_vec(&shape);

    let mut expected = common::VecBuf::default();
    let mut s = udigest::encoding::EncodeValue::new(&mut expected)
        .encode_enum()
        .with_variant("Generic");
    s.add_field("name").encode_leaf_value("triangle");
    s.add_field("wrapper").encode_leaf_value([10, 11, 100]);
    s.add_field("points").encode(&[(0u8, 0u8), (1, 0), (0, 1)]);
    s.finish();

    assert_eq!(hex::encode(actual), hex::encode(expected.0));
}
