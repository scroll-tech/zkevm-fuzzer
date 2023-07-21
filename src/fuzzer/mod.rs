use std::collections::BTreeMap;

use crate::test::ErasedCircuitTestBuilder;
use erased_serde::Serialize;
use once_cell::sync::Lazy;

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

pub trait FuzzerCaseGenerator: Send + Sync + 'static {
    fn name(&self) -> &'static str;
    fn gen_test_case(&self) -> FuzzerCase;
}

pub struct FuzzerWorker {

}

pub mod calldatacopy_root;

pub static FUZZERS: Lazy<BTreeMap<&'static str, Box<dyn FuzzerCaseGenerator>>> = Lazy::new(|| {
    let mut map = BTreeMap::<_, Box<dyn FuzzerCaseGenerator>>::new();
    let fuzzer = calldatacopy_root::Fuzzer;
    map.insert(fuzzer.name(), Box::new(fuzzer));
    map
});