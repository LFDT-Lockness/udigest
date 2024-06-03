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
    #[udigest(as_bytes = SomeValue::to_vec)]
    pub bytes_as_vec: SomeValue,
    #[udigest(skip)]
    pub ignored_field: Empty,
}

pub struct SomeValue(Vec<u8>);
impl SomeValue {
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }

    pub fn to_vec(&self) -> Vec<u8> {
        self.0.clone()
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
    Variant3 {},
    Variant4(),
    Variant5,
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

#[derive(udigest::Digestable)]
pub struct StructAttrWith {
    #[udigest(with = encoding::encode_bar)]
    foo: Bar,
}

pub struct Bar;

mod encoding {
    pub fn encode_bar<B: udigest::Buffer>(
        _bar: &super::Bar,
        encoder: udigest::encoding::EncodeValue<B>,
    ) {
        let mut list = encoder.encode_list();
        list.add_leaf().chain("foo");
        list.add_leaf().chain("bar");
        list.finish()
    }
}
