#[derive(udigest::Digestable)]
#[udigest(tag = "udigest.example.Person.v1")]
struct Person {
    name: String,
    #[udigest(rename = "job")]
    job_title: String,
}
