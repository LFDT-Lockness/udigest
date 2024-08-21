mod common;

#[test]
fn list_of_bytestrings() {
    #[derive(udigest::Digestable)]
    struct Post {
        title: String,
        content: String,
        #[udigest(as = Vec<udigest::Bytes>)]
        images: Vec<Vec<u8>>,
    }

    impl Post {
        fn digest_expected(&self) -> impl udigest::Digestable + '_ {
            udigest::inline_struct!({
                title: &self.title,
                content: &self.content,
                images: self.images
                    .clone()
                    .into_iter()
                    .map(udigest::Bytes)
                    .collect::<Vec<_>>(),
            })
        }
    }

    let post = Post {
        title: "My first post!".into(),
        content: "This is the first post I've ever written!".into(),
        images: vec![b"some image".to_vec()],
    };

    let expected = common::encode_to_vec(&post.digest_expected());
    let actual = common::encode_to_vec(&post);

    assert_eq!(hex::encode(expected), hex::encode(actual));
}

#[test]
fn hash_map() {
    #[derive(udigest::Digestable)]
    struct Attributes(
        #[udigest(as = std::collections::BTreeMap<_, udigest::Bytes>)]
        std::collections::HashMap<String, Vec<u8>>,
    );

    #[derive(udigest::Digestable)]
    struct EncodingExpected(std::collections::BTreeMap<String, udigest::Bytes<Vec<u8>>>);

    impl Attributes {
        fn digest_expected(&self) -> impl udigest::Digestable + '_ {
            let encoding = self
                .0
                .iter()
                .map(|(k, v)| (k.clone(), udigest::Bytes(v.clone())))
                .collect::<std::collections::BTreeMap<_, _>>();
            EncodingExpected(encoding)
        }
    }

    let attrs = Attributes(FromIterator::from_iter([
        ("some_attr".to_string(), b"value1".to_vec()),
        ("attr".to_string(), b"value2".to_vec()),
        ("some_other_attr".to_string(), b"value3".to_vec()),
    ]));

    let expected = common::encode_to_vec(&attrs.digest_expected());
    let actual = common::encode_to_vec(&attrs);

    assert_eq!(hex::encode(expected), hex::encode(actual));
}

#[test]
fn option() {
    #[derive(udigest::Digestable)]
    struct Person {
        nickname: String,
        #[udigest(as = Option<udigest::Bytes>)]
        avatar: Option<Vec<u8>>,
    }

    impl Person {
        fn digest_expected(&self) -> impl udigest::Digestable + '_ {
            udigest::inline_struct!({
                nickname: &self.nickname,
                avatar: self.avatar.as_ref().map(udigest::Bytes)
            })
        }
    }

    let person = Person {
        nickname: "c00l_name".to_string(),
        avatar: Some(b"image".to_vec()),
    };

    let expected = common::encode_to_vec(&person.digest_expected());
    let actual = common::encode_to_vec(&person);

    assert_eq!(hex::encode(expected), hex::encode(actual));
}
