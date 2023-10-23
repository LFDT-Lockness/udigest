#[derive(udigest::Digestable)]
#[udigest(tag = concat!("udigest.example", ".v1"))]
pub struct DigestableExample {
    pub some_string: String,
    pub integer: u64,
    pub list: Vec<String>,
    #[udigest(as_bytes)]
    pub bytes: [u8; 10],
    #[udigest(as_bytes = SomeValue::as_bytes)]
    pub more_bytes: SomeValue,
    #[udigest(skip)]
    pub ignored_field: Empty,
}

pub struct SomeValue(Vec<u8>);
impl SomeValue {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

pub struct Empty;
