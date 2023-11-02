#[derive(udigest::Digestable)]
#[udigest(tag = concat!("udigest.example", ".v1"))]
pub struct DigestableExample {
    pub some_string: String,
    pub integer: u64,
    pub list: Vec<String>,
    #[udigest(as_bytes)]
    pub bytes: [u8; 10],
    #[udigest(as_bytes = SomeValue::as_bytes)]
    #[udigest(rename = "more more bytes")]
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
        #[udigest(rename = 2_u32.to_be_bytes())]
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
#[udigest(bound = "")]
pub struct Bounds<D>
where
    D: Clone,
{
    _ph: std::marker::PhantomData<D>,
}

#[derive(udigest::Digestable)]
pub enum EmptyEnum {}

pub mod isolated {
    use ::udigest as udigest2;
    mod udigest {}

    #[derive(udigest2::Digestable)]
    #[udigest(root = udigest2)]
    pub struct Foo {
        bar: String,
    }
}

#[derive(udigest::Digestable)]
#[udigest(tag = "udigest.example.v1")]
pub enum EnumWithTag {
    Variant1(String),
    Variant2 { int: u32 },
}
