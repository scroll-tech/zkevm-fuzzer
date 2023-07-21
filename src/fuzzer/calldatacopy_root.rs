use crate::fuzzer::{Fuzzer, FuzzerCase};
use crate::input::opcodes::calldatacopy::{CalldataCopyRootArgs, MAX_CALLDATA_LENGTH};
use crate::input::FromRng;
use crate::test::ErasedCircuitTestBuilder;
use bus_mapping::circuit_input_builder::CircuitsParams;
use eth_types::bytecode;
use mock::test_ctx::helpers::account_0_code_account_1_no_code;
use mock::TestContext;

struct CalldataCopyRootFuzzer;

impl Fuzzer for CalldataCopyRootFuzzer {
    fn name(&self) -> &'static str {
        "calldatacopy-root"
    }

    fn gen_test_case(&self) -> FuzzerCase {
        let mut rng = rand::thread_rng();
        let args = CalldataCopyRootArgs::from_rng(&mut rng);

        let bytecode = bytecode! {
            PUSH32(args.length)
            PUSH32(args.data_offset)
            PUSH32(args.memory_offset)
            #[start]
            CALLDATACOPY
            STOP
        };

        let ctx = TestContext::<2, 1>::new(
            None,
            account_0_code_account_1_no_code(bytecode),
            |mut txs, accs| {
                txs[0]
                    .from(accs[1].address)
                    .to(accs[0].address)
                    .input(args.calldata.0.clone().into());
            },
            |block, _tx| block.number(0xcafeu64),
        )
        .unwrap();

        FuzzerCase::new(
            Box::new(args),
            ErasedCircuitTestBuilder::new_from_test_ctx(ctx).params(CircuitsParams {
                max_calldata: MAX_CALLDATA_LENGTH,
                ..CircuitsParams::default()
            }),
        )
    }
}

#[test]
fn test_calldatacopy_root() {
    let case = CalldataCopyRootFuzzer.gen_test_case();
    case.test_builder.run_catch().unwrap();
}