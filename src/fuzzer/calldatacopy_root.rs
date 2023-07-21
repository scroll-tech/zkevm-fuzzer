use crate::fuzzer::{Fuzzer, FuzzerCase};
use crate::input::opcodes::calldatacopy::{CalldataCopyRootArgs, MAX_CALLDATA_LENGTH};
use crate::input::FromRng;
use crate::test::ErasedCircuitTestBuilder;
use bus_mapping::circuit_input_builder::CircuitsParams;
use eth_types::bytecode;
use mock::test_ctx::helpers::account_0_code_account_1_no_code;
use mock::TestContext;

pub struct CalldataCopyRootFuzzer;

impl Fuzzer for CalldataCopyRootFuzzer {
    fn name(&self) -> &'static str {
        "calldatacopy-root"
    }

    fn gen_test_case(&self) -> FuzzerCase {
        let mut rng = rand::thread_rng();
        let args = CalldataCopyRootArgs::from_rng(&mut rng);

        let bytecode = bytecode! {
            PUSH32(args.length.0)
            PUSH32(args.data_offset.0)
            PUSH32(args.memory_offset.0)
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
                max_rws: 64,
                max_rlp_rows: 64,
                max_copy_rows: MAX_CALLDATA_LENGTH * 8,
                max_inner_blocks: 0,
                max_exp_steps: 0,
                max_bytecode: 128,
                max_calldata: MAX_CALLDATA_LENGTH,
                max_mpt_rows: 0,
                ..CircuitsParams::default()
            }),
        )
    }
}