// regular struct

#[derive(udigest::Digestable)]
struct Employee {
    name: String,
    competences: Vec<Competence>,
}

enum Competence {
    Engineer,
    Cryptographer,
    Hardware,
}

// tuple struct

#[derive(udigest::Digestable)]
struct Stageur(Competence);

// this will have a weird span in rendered error, but there's nothing we can do
// at this time

#[derive(udigest::Digestable)]
struct PhdSummerStudent(Vec<Competence>);

fn main() {}
