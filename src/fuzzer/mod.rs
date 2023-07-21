use arbitrary::Arbitrary;

pub trait Fuzzer<I: Arbitrary> {

}

#[test]
fn as_trait_obj() {
    struct Foo;
    impl Fuzzer<u8> for Foo {}

    let _ = Box::new(Foo as dyn Fuzzer);
}

mod calldatacopy_root;