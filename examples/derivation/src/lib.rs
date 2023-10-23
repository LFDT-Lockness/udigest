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

#[derive(udigest::Digestable)]
pub enum EnumExample {
    Variant1 {
        integer: i32,
        string: String,
        #[udigest(as_bytes = SomeValue::as_bytes)]
        something_else: SomeValue,
    },
    Variant2(String, #[udigest(as_bytes)] Vec<u8>, #[udigest(skip)] Empty),
    Vartiant3 {},
    Variant4(),
    Vartiant5,
}

#[derive(udigest::Digestable)]
pub enum EmptyEnum {}
