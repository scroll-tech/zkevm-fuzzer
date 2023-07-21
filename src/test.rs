use bus_mapping::circuit_input_builder::CircuitsParams;
use bus_mapping::mock::BlockData;
use eth_types::geth_types::GethData;
use halo2_proofs::dev::{MockProver, VerifyFailure};
use halo2_proofs::halo2curves::bn256::Fr;
use mock::TestContext;
use std::panic;
use zkevm_circuits::copy_circuit::CopyCircuit;
use zkevm_circuits::evm_circuit::EvmCircuit;
use zkevm_circuits::state_circuit::StateCircuit;
use zkevm_circuits::util::{log2_ceil, SubCircuit};
use zkevm_circuits::witness::{block_convert, Block, Rw};

type CheckFunction =
    dyn Fn(MockProver<Fr>, &Vec<usize>, &Vec<usize>) -> Result<(), Vec<VerifyFailure>>;

pub struct ErasedCircuitTestBuilder {
    test_ctx: Option<ErasedTestContext>,
    circuits_params: Option<CircuitsParams>,
    block: Option<Block<Fr>>,
    evm_checks: Box<CheckFunction>,
    state_checks: Box<CheckFunction>,
    copy_checks: Box<CheckFunction>,
    block_modifiers: Vec<Box<dyn Fn(&mut Block<Fr>)>>,
}

impl ErasedCircuitTestBuilder {
    pub fn empty() -> Self {
        ErasedCircuitTestBuilder {
            test_ctx: None,
            circuits_params: None,
            block: None,
            evm_checks: Box::new(|prover, gate_rows, lookup_rows| {
                prover.verify_at_rows_par(gate_rows.iter().cloned(), lookup_rows.iter().cloned())
            }),
            state_checks: Box::new(|prover, gate_rows, lookup_rows| {
                prover.verify_at_rows_par(gate_rows.iter().cloned(), lookup_rows.iter().cloned())
            }),
            copy_checks: Box::new(|prover, gate_rows, lookup_rows| {
                prover.verify_at_rows_par(gate_rows.iter().cloned(), lookup_rows.iter().cloned())
            }),
            block_modifiers: vec![],
        }
    }

    /// Generates a CTBC from a [`TestContext`] passed with all the other fields
    /// set to [`Default`].
    pub fn new_from_test_ctx<const NACC: usize, const NTX: usize>(
        ctx: TestContext<NACC, NTX>,
    ) -> Self {
        Self::empty().test_ctx(ctx)
    }

    /// Generates a CTBC from a [`Block`] passed with all the other fields
    /// set to [`Default`].
    pub fn new_from_block(block: Block<Fr>) -> Self {
        Self::empty().block(block)
    }

    /// Allows to produce a [`TestContext`] which will serve as the generator of
    /// the Block.
    pub fn test_ctx<const NACC: usize, const NTX: usize>(
        mut self,
        ctx: TestContext<NACC, NTX>,
    ) -> Self {
        self.test_ctx = Some(ctx.into());
        self
    }

    /// Allows to pass a non-default [`CircuitsParams`] to the builder.
    /// This means that we can increase for example, the `max_rws` or `max_txs`.
    pub fn params(mut self, params: CircuitsParams) -> Self {
        assert!(
            self.block.is_none(),
            "circuit_params already provided in the block"
        );
        self.circuits_params = Some(params);
        self
    }

    /// Allows to pass a [`Block`] already built to the constructor.
    pub fn block(mut self, block: Block<Fr>) -> Self {
        self.block = Some(block);
        self
    }

    /// Allows to provide checks different than the default ones for the State
    /// Circuit verification.
    pub fn state_checks(mut self, state_checks: Box<CheckFunction>) -> Self {
        self.state_checks = state_checks;
        self
    }

    /// Allows to provide checks different than the default ones for the EVM
    /// Circuit verification.
    pub fn evm_checks(mut self, evm_checks: Box<CheckFunction>) -> Self {
        self.evm_checks = evm_checks;
        self
    }

    /// Allows to provide modifier functions for the [`Block`] that will be
    /// generated within this builder.
    ///
    /// That removes the need in a lot of tests to build the block outside of
    /// the builder because they need to modify something particular.
    pub fn block_modifier(mut self, modifier: Box<dyn Fn(&mut Block<Fr>)>) -> Self {
        self.block_modifiers.push(modifier);
        self
    }
}

impl ErasedCircuitTestBuilder {
    /// Triggers the `CircuitTestBuilder` to convert the [`TestContext`] if any,
    /// into a [`Block`] and apply the default or provided block_modifiers or
    /// circuit checks to the provers generated for the State and EVM circuits.
    pub fn run_catch(self) -> Result<(), Vec<VerifyFailure>> {
        let params = if let Some(block) = self.block.as_ref() {
            block.circuits_params
        } else {
            self.circuits_params.unwrap_or_default()
        };
        debug!("params in CircuitTestBuilder: {:?}", params);

        let block: Block<Fr> = if self.block.is_some() {
            self.block.unwrap()
        } else if self.test_ctx.is_some() {
            let block: GethData = self.test_ctx.unwrap().to_geth_data();
            let mut builder = BlockData::new_from_geth_data_with_params(block.clone(), params)
                .new_circuit_input_builder();
            builder
                .handle_block(&block.eth_block, &block.geth_traces)
                .unwrap();
            // Build a witness block from trace result.
            let mut block = block_convert(&builder.block, &builder.code_db).unwrap();

            for modifier_fn in self.block_modifiers {
                modifier_fn.as_ref()(&mut block);
            }
            block
        } else {
            panic!("No attribute to build a block was passed to the CircuitTestBuilder")
        };

        const NUM_BLINDING_ROWS: usize = 64;
        // Run evm circuit test
        {
            let k = block.get_test_degree();
            let (active_gate_rows, active_lookup_rows) = EvmCircuit::<Fr>::get_active_rows(&block);

            let circuit = EvmCircuit::get_test_cicuit_from_block(block.clone());
            let prover = MockProver::<Fr>::run(k, &circuit, vec![]).unwrap();

            self.evm_checks.as_ref()(prover, &active_gate_rows, &active_lookup_rows)?
        }

        // Run state circuit test
        {
            let rows_needed = StateCircuit::<Fr>::min_num_rows_block(&block).1;
            let k = log2_ceil(rows_needed + NUM_BLINDING_ROWS);
            let state_circuit = StateCircuit::<Fr>::new(block.rws.clone(), params.max_rws);
            let instance = state_circuit.instance();
            let prover = MockProver::<Fr>::run(k, &state_circuit, instance).unwrap();
            // Skip verification of Start rows to accelerate testing
            let non_start_rows_len = state_circuit
                .rows
                .iter()
                .filter(|rw| !matches!(rw, Rw::Start { .. }))
                .count();
            let rows = (params.max_rws - non_start_rows_len..params.max_rws).collect();

            self.state_checks.as_ref()(prover, &rows, &rows)?;
        }

        // Run copy circuit test
        {
            let (active_rows, max_rows) = CopyCircuit::<Fr>::min_num_rows_block(&block);
            let k1 = block.get_test_degree();
            let k2 = log2_ceil(max_rows + NUM_BLINDING_ROWS);
            let k = k1.max(k2);
            let copy_circuit = CopyCircuit::<Fr>::new_from_block(&block);
            let instance = copy_circuit.instance();
            let prover = MockProver::<Fr>::run(k, &copy_circuit, instance).unwrap();
            let rows = (0..active_rows).collect();

            self.copy_checks.as_ref()(prover, &rows, &rows)?;
        }

        Ok(())
    }
}

pub struct ErasedTestContext(Box<dyn TestContextType>);

pub trait TestContextType {
    fn to_geth_data(&self) -> GethData;
}

impl<const NACC: usize, const NTX: usize> TestContextType for TestContext<NACC, NTX> {
    fn to_geth_data(&self) -> GethData {
        GethData {
            chain_id: self.chain_id,
            history_hashes: self.history_hashes.clone(),
            eth_block: self.eth_block.clone(),
            geth_traces: self.geth_traces.clone(),
            accounts: self.accounts.to_vec(),
        }
    }
}

impl<const NACC: usize, const NTX: usize> From<TestContext<NACC, NTX>> for ErasedTestContext {
    fn from(ctx: TestContext<NACC, NTX>) -> Self {
        Self(Box::new(ctx))
    }
}

impl ErasedTestContext {
    pub fn to_geth_data(&self) -> GethData {
        self.0.to_geth_data()
    }
}
