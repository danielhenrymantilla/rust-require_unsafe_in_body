#![cfg(not(feature = "unit-tests"))]

#[test]
fn ui ()
{
    ::trybuild::TestCases::new()
        .compile_fail("tests/ui/*.rs")
    ;
}
