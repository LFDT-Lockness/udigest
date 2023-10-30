use sha2::Sha256;

#[derive(udigest::Digestable)]
#[udigest(tag = "udigest.example.Person.v1")]
struct Person {
    name: String,
    #[udigest(rename = "job")]
    job_title: String,
}

fn main() {
    let person = Person {
        name: "Alice".into(),
        job_title: "cryptographer".into(),
    };

    let hash = udigest::Unambiguous::<Sha256>::with_tag("udigest.example").digest(&person);
    println!("{}", hex::encode(hash));
}
