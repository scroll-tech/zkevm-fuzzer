use tracing_subscriber::{EnvFilter, fmt, prelude::*};
use zkevm_fuzzer::fuzzer::{calldatacopy_root::Fuzzer, FuzzerCaseGenerator};

fn main() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env())
        .init();
    let case = Fuzzer.gen_test_case();
    case.test_builder.run_catch().unwrap();
}