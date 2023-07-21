use crate::test::ErasedCircuitTestBuilder;
use erased_serde::Serialize;

pub struct FuzzerCase {
    pub input: Box<dyn Serialize>,
    pub test_builder: ErasedCircuitTestBuilder,
}

impl FuzzerCase {
    pub fn new(input: Box<dyn Serialize>, test_builder: ErasedCircuitTestBuilder) -> Self {
        Self {
            input,
            test_builder,
        }
    }
}

pub trait Fuzzer {
    fn name(&self) -> &'static str;
    fn gen_test_case(&self) -> FuzzerCase;
}

mod calldatacopy_root;
