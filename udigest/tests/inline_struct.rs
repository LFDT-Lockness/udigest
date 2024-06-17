#[test]
fn no_tag() {
    #[derive(udigest::Digestable)]
    struct Person {
        name: &'static str,
        age: u32,
    }

    let hash_expected = udigest::hash::<sha2::Sha256>(&Person {
        name: "Alice",
        age: 24,
    });

    let hash_actual = udigest::hash::<sha2::Sha256>(&udigest::inline_struct!({
        name: "Alice",
        age: 24_u32,
    }));

    assert_eq!(hex::encode(hash_expected), hex::encode(hash_actual));
}

#[test]
fn with_tag() {
    #[derive(udigest::Digestable)]
    #[udigest(tag = "some_tag")]
    struct Person {
        name: &'static str,
        age: u32,
    }

    let hash_expected = udigest::hash::<sha2::Sha256>(&Person {
        name: "Alice",
        age: 24,
    });

    let hash_actual = udigest::hash::<sha2::Sha256>(&udigest::inline_struct!("some_tag" {
        name: "Alice",
        age: 24_u32,
    }));

    assert_eq!(hex::encode(hash_expected), hex::encode(hash_actual));
}

#[test]
fn embedded_structs() {
    #[derive(udigest::Digestable)]
    struct Person {
        name: &'static str,
        age: u32,
        preferences: Preferences,
    }
    #[derive(udigest::Digestable)]
    struct Preferences {
        display_email: bool,
        receive_newsletter: bool,
    }

    let hash_expected = udigest::hash::<sha2::Sha256>(&Person {
        name: "Alice",
        age: 24,
        preferences: Preferences {
            display_email: false,
            receive_newsletter: false,
        },
    });

    let hash_actual = udigest::hash::<sha2::Sha256>(&udigest::inline_struct!({
        name: "Alice",
        age: 24_u32,
        preferences: udigest::inline_struct!({
            display_email: false,
            receive_newsletter: false,
        })
    }));

    assert_eq!(hex::encode(hash_expected), hex::encode(hash_actual));
}

#[test]
fn shorter_syntax() {
    let name = "Alice";
    let age = 24_u32;
    let alice1 = udigest::inline_struct!({
        name,
        age,
    });

    let name = name.to_owned();
    let alice2 = udigest::inline_struct!({
        &name,
        age,
    });

    assert_eq!(
        udigest::hash::<sha2::Sha256>(&alice1),
        udigest::hash::<sha2::Sha256>(&alice2),
    );
    drop(alice2);

    // `name` is not consumed
    drop(name);
}
