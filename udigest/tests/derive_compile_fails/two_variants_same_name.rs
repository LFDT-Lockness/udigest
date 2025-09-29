// variant name duplication may occur by renaming one of the variants:
#[derive(udigest::Digestable)]
enum Employee1 {
    Dev {
        stack: String,
    },
    #[udigest(rename = "Dev")]
    Cryptographer {
        field: String,
    },
    Hr {
        head: bool,
    },
}

// ... by renaming one of the variants to byte string:
#[derive(udigest::Digestable)]
enum Employee2 {
    Dev {
        stack: String,
    },
    #[udigest(rename = b"Dev")]
    Cryptographer {
        field: String,
    },
    Hr {
        head: bool,
    },
}

// or by renaming two variants:
#[derive(udigest::Digestable)]
enum Employee3 {
    #[udigest(rename = "SomethingElse")]
    Dev {
        stack: String,
    },
    #[udigest(rename = "SomethingElse")]
    Cryptographer {
        field: String,
    },
    Hr {
        head: bool,
    },
}

// ... one of them may be a bytestring:
#[derive(udigest::Digestable)]
enum Employee4 {
    #[udigest(rename = "SomethingElse")]
    Dev {
        stack: String,
    },
    #[udigest(rename = b"SomethingElse")]
    Cryptographer {
        field: String,
    },
    Hr {
        head: bool,
    },
}
#[derive(udigest::Digestable)]
enum Employee5 {
    #[udigest(rename = b"SomethingElse")]
    Dev {
        stack: String,
    },
    #[udigest(rename = "SomethingElse")]
    Cryptographer {
        field: String,
    },
    Hr {
        head: bool,
    },
}

// or they both can be a bytestring
#[derive(udigest::Digestable)]
enum Employee6 {
    #[udigest(rename = b"SomethingElse")]
    Dev {
        stack: String,
    },
    #[udigest(rename = b"SomethingElse")]
    Cryptographer {
        field: String,
    },
    Hr {
        head: bool,
    },
}

fn main() {}
