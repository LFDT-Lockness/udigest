#[derive(udigest::Digestable)]
pub struct Person {
    name: &'static str,
    age: u16,
    job_title: &'static str,
}

const ALICE: Person = Person {
    name: "Alice",
    age: 24,
    job_title: "cryptographer",
};

const BOB: Person = Person {
    name: "Bob",
    age: 25,
    job_title: "research engineer",
};

#[test]
fn sha2_256() {
    let alice_hash = udigest::hash::<sha2::Sha256>(&ALICE);
    assert_eq!(
        hex::encode(alice_hash.as_slice()),
        "99e258d6a6ccc430a50dcbf4e9c8cfb59ad0b94b96b83f0182a9a68eb1c5438f",
    );

    let bob_hash = udigest::hash::<sha2::Sha256>(&BOB);
    assert_eq!(
        hex::encode(bob_hash.as_slice()),
        "28474b5dec79b222b74badc2d78f9f81c0fbfd1ee04a134947cd07f44237ade3",
    );
}

#[test]
fn shake256() {
    use digest::XofReader;

    let mut hash = [0u8; 123];
    let mut alice_hash_reader = udigest::hash_xof::<sha3::Shake256>(&ALICE);
    alice_hash_reader.read(&mut hash);
    assert_eq!(
        hex::encode(&hash),
        "54809cf7b06438f9508785fb5e46bdfd7714b39b026e86fa7cc8a8442ae10bd5\
        49baeced19ff0642b042ae4e92636536baec5748dad99e71fc53a4361734973ae\
        2c4f1547305a76addd5b6076509ddbf91bd5beb71ba09598e265704d1e9a1c0c3\
        5fae7f8e4958ceb38962fc8e6fc56e32bef4e88f64bc8a88f88a"
    );

    let mut bob_hash_reader = udigest::hash_xof::<sha3::Shake256>(&BOB);
    bob_hash_reader.read(&mut hash);
    assert_eq!(
        hex::encode(&hash),
        "f68ca9eeb7e09657fc54a5cbbd50acdd6d9fccd29ec1a3eb460b673ea59d64a9\
        b2ec8be97c7d7858ad6724cf8c27299569bd72193c77bb339883214a4477c0762\
        f9cf31a2d698562f57dff5ede03d6928feba694975445e7dabe3d67e67b710f26\
        11f4f14471917bd447d199c32eb93dbcaf1fdbefe05132911991"
    );
}

#[test]
fn blake2b() {
    let mut out = [0u8; 63];

    udigest::hash_vof::<blake2::Blake2bVar>(&ALICE, &mut out).unwrap();
    assert_eq!(
        hex::encode(&out),
        "91d1ce144fd46ed5400895c8db5f2b39c95870020c6627af034a9fa09c2f2cc3\
        f4c8c7d4e8d38ff16e4f54360b4387c0439cf30c51c21c78f904cda9205023"
    );

    udigest::hash_vof::<blake2::Blake2bVar>(&BOB, &mut out).unwrap();
    assert_eq!(
        hex::encode(&out),
        "2f916c687c82c0f37d31df061c0453e98d0655e1877d4a55ec1507514822a2c4\
        b7cac3ca66a5e3deb678f915210e93f2fc14591b987f121083623ab024ece4"
    );
}
