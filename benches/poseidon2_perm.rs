use std::fs;
use anyhow::Result;
use std::time::Instant;
use crate::circuits::BaseCircuit;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
// use env_logger::builder;
use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hash_types::RichField;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::circuit_data::{CircuitConfig, CircuitData};
use plonky2::plonk::config::{AlgebraicHasher, GenericConfig, Hasher, PoseidonGoldilocksConfig};
use plonky2_poseidon2::config::Poseidon2GoldilocksConfig;
// use plonky2_monolith::gates::generate_config_for_monolith_gate;
// use plonky2_monolith::monolith_hash::monolith_goldilocks::MonolithGoldilocksConfig;
// use plonky2_monolith::monolith_hash::{Monolith, MonolithHash};
use tynm::type_name;
use plonky2::hash::hashing::PlonkyPermutation;
// use plonky2::hash::poseidon2::Poseidon2Hash;
use plonky2_poseidon2::poseidon2_hash::poseidon2::{Poseidon2, Poseidon2Hash};
use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartialWitness, WitnessWrite};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use std::marker::PhantomData;
use plonky2::plonk::proof::ProofWithPublicInputs;

mod circuits;

macro_rules! pretty_print {
    ($($arg:tt)*) => {
        print!("\x1b[0;36mINFO ===========>\x1b[0m ");
        println!($($arg)*);
    }
}

pub struct PoseidonCircuit<
    F: RichField + Extendable<D> + Poseidon2,
    C: GenericConfig<D, F = F>,
    const D: usize,
    H: Hasher<F> + AlgebraicHasher<F>,
> {
    public_input: Vec<Target>,
    // public_output: Vec<Target>,
    circuit_data: CircuitData<F, C, D>,
    num_powers: usize,
    _hasher: PhantomData<H>,
}

impl<
    F: RichField + Extendable<D> + Poseidon2,
    C: GenericConfig<D, F = F>,
    const D: usize,
    H: Hasher<F> + AlgebraicHasher<F>,
> PoseidonCircuit<F, C, D, H>
{
    pub fn build_circuit(config: CircuitConfig, log_num_hashes: usize) -> Self {
        let num_hashes: usize = 1usize << log_num_hashes;

        // let mut builder = CircuitBuilder::<F, D>::new(config);
        // let mut res_t = builder.add_virtual_public_input();
        // let init_t = res_t;
        // let zero = builder.zero();
        // let to_be_hashed_t = builder.add_virtual_target();
        // for _ in 0..num_hashes {
        //     res_t = builder.mul(res_t, init_t);
        //     res_t = builder.hash_n_to_m_no_pad::<H>(vec![res_t, to_be_hashed_t, zero, zero], 1)[0];
        // }
        //
        // let out_t = builder.add_virtual_public_input();
        // let is_eq_t = builder.is_equal(out_t, res_t);
        // builder.assert_one(is_eq_t.target);
        //
        // let data = builder.build::<C>();

        const T: usize = 12;

        let mut builder = CircuitBuilder::<F, D>::new(config);
        let zero = builder.zero();
        let mut state = H::AlgebraicPermutation::new(core::iter::repeat(zero));

        let mut initial = Vec::new(); // vec![];
        for _ in 0..T {
            let x = builder.add_virtual_public_input();
            initial.push(x);
        }

        state.set_from_slice(&initial, 0);
        // let _false = CircuitBuilder::_false();
        // let false = BoolTarget::new_unsafe(self.zero())
        // let N = 1;
        for k in 0..num_hashes {
            state = builder.permute::<H>(state);
        }

        let output = state.squeeze();
        for o in output{
            builder.register_public_input(*o);
        }

        // for j in 0..T {
        //     builder.register_public_input(state[j]);
        // }
        // H::permute_swapped()
        let data = builder.build::<C>();

        // pretty_print!(
        //     "circuit size: 2^{} gates",
        //     data.common.degree_bits()
        // );

        Self {
            public_input: initial,
            // public_output: output,
            circuit_data: data,
            num_powers: num_hashes,
            _hasher: PhantomData::<H>,
        }
    }

    pub fn generate_proof(&self, init: F) -> Result<ProofWithPublicInputs<F, C, D>> {
        // let mut pw = PartialWitness::<F>::new();
        //
        // pw.set_target(self.public_input, init);
        // let to_be_hashed = F::rand();
        // pw.set_target(self.private_input, to_be_hashed);
        // let mut res = init;
        // for _ in 0..self.num_powers {
        //     res = res.mul(init);
        //     res =
        //         hash_n_to_m_no_pad::<_, H::Permutation>(&[res, to_be_hashed, F::ZERO, F::ZERO], 1)
        //             [0];
        // }
        //
        // pw.set_target(self.public_output, res);
        //
        // let proof = self.circuit_data.prove(pw)?;
        //
        // self.circuit_data.verify(proof.clone())?;
        //
        // assert_eq!(proof.public_inputs[1], res);
        const T: usize = 12;

        let mut pw = PartialWitness::<F>::new();
        for j in 0..T {
            pw.set_target(self.public_input[j], F::from_canonical_usize(j));
        }

        let proof = self.circuit_data.prove(pw).unwrap();

        Ok(proof)
    }

    pub fn get_circuit_data(&self) -> &CircuitData<F, C, D> {
        &self.circuit_data
    }
}

fn bench_poseidon2_perm<
    F: RichField + Extendable<D> + Poseidon2,
    const D: usize,
    C: GenericConfig<D, F = F>,
    H: Hasher<F> + AlgebraicHasher<F>,
>(
    c: &mut Criterion,
    config: CircuitConfig,
) {

    let mut group = c.benchmark_group(&format!(
        "poseidon-proof<{}, {}>",
        type_name::<C>(),
        type_name::<H>()
    ));

    for log_num_hashes in [ 10, 11, 12, 13 ] {
        group.bench_function(
            format!("build circuit for 2^{} permutations", log_num_hashes).as_str(),
            |b| {
                b.iter_with_large_drop(|| {
                    PoseidonCircuit::<F, C, D, H>::build_circuit(config.clone(), log_num_hashes);
                })
            },
        );

        let poseidon_circuit =
            PoseidonCircuit::<F, C, D, H>::build_circuit(config.clone(), log_num_hashes);

        pretty_print!(
            "circuit size: 2^{} gates",
            poseidon_circuit.get_circuit_data().common.degree_bits()
        );

        group.bench_function(
            format!("prove circuit with 2^{} permutations", log_num_hashes).as_str(),
            |b| {
                b.iter_batched(
                    || F::rand(),
                    |init| poseidon_circuit.generate_proof(init).unwrap(),
                    BatchSize::PerIteration,
                )
            },
        );

        let proof = poseidon_circuit.generate_proof(F::rand()).unwrap();

        pretty_print!("proof size: {}", proof.to_bytes().len());

        group.bench_function(
            format!("verify circuit with 2^{} permutations", log_num_hashes).as_str(),
            |b| {
                b.iter_batched(
                    || (poseidon_circuit.get_circuit_data(), proof.clone()),
                    |(data, proof)| data.verify(proof).unwrap(),
                    BatchSize::PerIteration,
                )
            },
        );
    }

    group.finish();

    // const T: usize = 12;
    //
    // let mut builder = CircuitBuilder::<F, D>::new(config);
    // let zero = builder.zero();
    // let mut state = H::AlgebraicPermutation::new(core::iter::repeat(zero));
    //
    // let mut initial = Vec::new(); // vec![];
    // for _ in 0..T {
    //     let x = builder.add_virtual_public_input();
    //     initial.push(x);
    // }
    //
    // state.set_from_slice(&initial, 0);
    //
    // let N = 1;
    // for k in 0..N {
    //     state = builder.permute::<H>(state);
    // }
    //
    // let output = state.squeeze();
    // for o in output{
    //     builder.register_public_input(*o);
    // }
    //
    // let data = builder.build::<C>();
    //
    // pretty_print!(
    //     "circuit size: 2^{} gates",
    //     data.common.degree_bits()
    // );
    //
    // let mut pw = PartialWitness::<F>::new();
    // for j in 0..T {
    //     pw.set_target(initial[j], F::from_canonical_usize(j));
    // }
    //
    // let time1 = Instant::now();
    // let proof = data.prove(pw).unwrap();
    // println!("proving the circuit: {:?}", time1.elapsed() );
    // let r = data.verify(proof.clone());
    // assert!(r.is_ok());
    //
    // for j in 0..T {
    //     println!("inputs[{}]  = {}", j, proof.public_inputs[j] );
    // }
    // for j in 0..output.len() {
    //     println!("outputs[{}]  = {}", j, proof.public_inputs[j+T] );
    // }
}

fn benchmark(c: &mut Criterion) {
    const D: usize = 2;
    type F = GoldilocksField;

    bench_poseidon2_perm::<F, D, PoseidonGoldilocksConfig, PoseidonHash>(
        c,
        CircuitConfig::standard_recursion_config(),
    );

    bench_poseidon2_perm::<F, D, Poseidon2GoldilocksConfig, Poseidon2Hash>(
        c,
        CircuitConfig::standard_recursion_config(),
    );
}

criterion_group!(name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark);
criterion_main!(benches);
