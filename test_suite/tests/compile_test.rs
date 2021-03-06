#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/fail/**/*.rs");
    t.pass("tests/ui/pass/*.rs");

    #[cfg(nightly)]
    {
        t.compile_fail("tests/ui/nightly/fail/**/*.rs");
        t.pass("tests/ui/nightly/pass/*.rs");
    }
}
