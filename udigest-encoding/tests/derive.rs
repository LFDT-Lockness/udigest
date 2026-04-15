#![allow(dead_code)]

#[derive(udigest_encoding::Digestable)]
struct Person {
    name: String,
}

use ::udigest_encoding as ue;

#[derive(ue::Digestable)]
struct PersonAlias {
    name: String,
}

use udigest_encoding::Digestable;

#[derive(Digestable)]
struct PersonImported {
    name: String,
}

struct VecBuffer(Vec<u8>);

impl udigest_encoding::encoding::Buffer for VecBuffer {
    fn write(&mut self, bytes: &[u8]) {
        self.0.extend_from_slice(bytes);
    }
}

#[test]
fn derive_works_with_all_root_resolution_modes() {
    let person = Person {
        name: "Alice".to_owned(),
    };

    let mut buf = VecBuffer(Vec::new());
    person.unambiguously_encode(udigest_encoding::encoding::EncodeValue::new(&mut buf));

    assert!(!buf.0.is_empty());
}
