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

    let hash = udigest::hash::<sha2::Sha256, _>(&person);
    println!("{}", hex::encode(hash));
}
