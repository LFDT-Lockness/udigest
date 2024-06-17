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
        "49c43095eaffb3e3232dd23940686d3c6fb80e5ff82b5a09d336ad32369ca9df",
    );

    let bob_hash = udigest::hash::<sha2::Sha256>(&BOB);
    assert_eq!(
        hex::encode(bob_hash.as_slice()),
        "3537c188336cb93f58df79149fb035fd132f23fac58a6e94d014178aeaa1c88e",
    );
}

#[test]
fn shake256() {
    use digest::XofReader;

    let mut hash = [0u8; 123];
    let mut alice_hash_reader = udigest::hash_xof::<sha3::Shake256>(&ALICE);
    alice_hash_reader.read(&mut hash);
    assert_eq!(
        hex::encode(hash),
        "ee629bcc426422887fe6f9a9a3384128bd5efc3c623a4599c8526c24a97972be\
        2a325ef03c95ac649b77f0193c901c942762e93fd939372ef484681220c6fc0b\
        0dc12be8c6b9ee914dac34697d0deeb3a3e510f24a1b0bfc24d144b639a66c6a\
        4c5a772b178eed159f87b581bb49aafdcdcca525fd57749aab6c32"
    );

    let mut bob_hash_reader = udigest::hash_xof::<sha3::Shake256>(&BOB);
    bob_hash_reader.read(&mut hash);
    assert_eq!(
        hex::encode(hash),
        "56cd71e796fc94176923b73bfe3f659ea7a9a666a2faae6020d1c4f41a51035a\
        e7965583087f1badf452a40036499d54075350d8e64e5b68b0f3f52c286c15e3\
        cb010249754a0c7f263d14c7a284da134ca133df84c62d80adfdb0ec0d5c3f0a\
        50e479dd025b27fb875c34ba72d9abc7a5990ce8c7f3c282dd6a0c"
    );
}

#[test]
fn blake2b() {
    let mut out = [0u8; 63];

    udigest::hash_vof::<blake2::Blake2bVar>(&ALICE, &mut out).unwrap();
    assert_eq!(
        hex::encode(out),
        "57b2a8a078ca3b04dc72b308696bc4715c62593b461608bff01388ef3bd49fed\
        244bd2e9407965ec2bfe13781ae3cd28ea0cb08fb4b46824ea7909c488fec8"
    );

    udigest::hash_vof::<blake2::Blake2bVar>(&BOB, &mut out).unwrap();
    assert_eq!(
        hex::encode(out),
        "83aa6240d105ec1b496e6963dbab3e48fd09860b734c963b59ee764781d922f1\
        207405232c1d84965b32f6a73b182b224d1533859f586c332377fe4a39489e"
    );
}
