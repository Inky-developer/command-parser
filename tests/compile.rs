#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/test_files/it_works.rs");
    t.pass("tests/test_files/multiple_options.rs");
    t.pass("tests/test_files/default_args.rs");
    t.pass("tests/test_files/captures.rs");
    t.compile_fail("tests/test_files/no_enum.rs");
}
