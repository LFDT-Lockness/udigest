// field name duplication may occur by renaming one of the fields:
#[derive(udigest::Digestable)]
struct Person1 {
    name: String,
    #[udigest(rename = "name")]
    surname: String,
}

// ... by renaming one of the fields to byte string:
#[derive(udigest::Digestable)]
struct Person2 {
    name: String,
    #[udigest(rename = b"name")]
    surname: String,
}

// or by renaming two fields:
#[derive(udigest::Digestable)]
struct Person3 {
    #[udigest(rename = "name")]
    first_name: String,
    #[udigest(rename = "name")]
    last_name: String,
}

// ... one of them may be a bytestring:
#[derive(udigest::Digestable)]
struct Person4 {
    #[udigest(rename = b"name")]
    first_name: String,
    #[udigest(rename = "name")]
    last_name: String,
}
#[derive(udigest::Digestable)]
struct Person5 {
    #[udigest(rename = "name")]
    first_name: String,
    #[udigest(rename = b"name")]
    last_name: String,
}

// or they both can be a bytestring
#[derive(udigest::Digestable)]
struct Person6 {
    #[udigest(rename = b"name")]
    first_name: String,
    #[udigest(rename = b"name")]
    last_name: String,
}

// in enum:
#[derive(udigest::Digestable)]
enum Shape {
    Circle {
        r: u16,
    },
    Rect {
        w: u16,
        #[udigest(rename = "w")]
        h: u16,
    },
    Square {
        w: u16,
    },
}

fn main() {}
