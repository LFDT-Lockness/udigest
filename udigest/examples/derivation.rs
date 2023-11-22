use sha2::Sha256;
use udigest::udigest;

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

    let tag = udigest::Tag::<Sha256>::new("udigest.example");
    let hash = udigest(tag, &person);
    println!("{}", hex::encode(hash));
}
