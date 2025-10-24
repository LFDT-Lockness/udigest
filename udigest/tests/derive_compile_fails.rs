#[test]
fn trybuild() {
    let t = trybuild::TestCases::new();
    t.compile_fail("./tests/derive_compile_fails/*.rs");
}
