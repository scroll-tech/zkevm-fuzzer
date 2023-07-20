#[macro_use] extern crate afl;

use bus_mapping::circuit_input_builder::CircuitsParams;
use eth_types::{bytecode, Word};
use mock::test_ctx::helpers::account_0_code_account_1_no_code;
use mock::TestContext;
use zkevm_circuits::test_util::CircuitTestBuilder;
use zkevm_fuzzer::opcodes::calldatacopy::{CalldataCopyRootArgs, MAX_CALLDATA_LENGTH};

fn main() {
    fuzz!(|args: CalldataCopyRootArgs| {
        let length = Word::from_big_endian(&args.length);
        let data_offset = Word::from_big_endian(&args.data_offset);
        let memory_offset = Word::from_big_endian(&args.memory_offset);
    
        let bytecode = bytecode! {
            PUSH32(length)
            PUSH32(data_offset)
            PUSH32(memory_offset)
            #[start]
            CALLDATACOPY
            STOP
        };
    
        // Get the execution steps from the external tracer
        let ctx = TestContext::<2, 1>::new(
            None,
            account_0_code_account_1_no_code(bytecode),
            |mut txs, accs| {
                txs[0]
                    .from(accs[1].address)
                    .to(accs[0].address)
                    .input(args.calldata.0.into());
            },
            |block, _tx| block.number(0xcafeu64),
        )
            .unwrap();
    
        CircuitTestBuilder::new_from_test_ctx(ctx)
            .params(CircuitsParams {
                max_calldata: MAX_CALLDATA_LENGTH,
                ..CircuitsParams::default()
            })
            .run();
    })
}