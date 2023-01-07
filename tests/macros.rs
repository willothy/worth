use casey::lower;
use worthc_macros::intrinsic_str;

#[test]
fn intrinsic_str_test() {
    assert_eq!(intrinsic_str!(lower!, test), "test");
    assert_eq!(intrinsic_str!(lower!, tesawdawd, "test2"), "test2");
}
