#![cfg(not(feature = "unit-tests"))]

#[test]
fn compile_fail ()
{
    ::trybuild::TestCases::new()
        .compile_fail("tests/compile_fail/*.rs")
    ;
}
