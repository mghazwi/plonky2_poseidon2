use std::fs;
use std::time::Instant;
use crate::circuits::BaseCircuit;
use criterion::{criterion_group, criterion_main, BatchSize, Criterion};
// use env_logger::builder;
use plonky2::field::extension::Extendable;
use plonky2::field::goldilocks_field::GoldilocksField;
use plonky2::hash::hash_types::RichField;
use plonky2::hash::poseidon::PoseidonHash;
use plonky2::plonk::circuit_data::CircuitConfig;
use plonky2::plonk::config::{AlgebraicHasher, GenericConfig, Hasher, PoseidonGoldilocksConfig};
use plonky2_poseidon2::config::Poseidon2GoldilocksConfig;
use tynm::type_name;
use plonky2::hash::hashing::PlonkyPermutation;
use plonky2_poseidon2::poseidon2_hash::poseidon2::{Poseidon2, Poseidon2Hash};
use plonky2::iop::witness::{PartialWitness, WitnessWrite};

mod circuits;

macro_rules! pretty_print {
    ($($arg:tt)*) => {
        print!("\x1b[0;36mINFO ===========>\x1b[0m ");
        println!($($arg)*);
    }
}

fn bench_base_proof<
    F: RichField + Extendable<D>,
    const D: usize,
    C: GenericConfig<D, F = F>,
    H: Hasher<F> + AlgebraicHasher<F>,
>(
    c: &mut Criterion,
    config: CircuitConfig,
) {
    let mut group = c.benchmark_group(&format!(
        "base-proof<{}, {}>",
        type_name::<C>(),
        type_name::<H>()
    ));

    for log_num_hashes in [ 15] {
        group.bench_function(
            format!("build circuit for 2^{} hashes", log_num_hashes).as_str(),
            |b| {
                b.iter_with_large_drop(|| {
                    BaseCircuit::<F, C, D, H>::build_base_circuit(config.clone(), log_num_hashes);
                })
            },
        );

        let base_circuit =
            BaseCircuit::<F, C, D, H>::build_base_circuit(config.clone(), log_num_hashes);

        pretty_print!(
            "circuit size: 2^{} gates",
            base_circuit.get_circuit_data().common.degree_bits()
        );

        group.bench_function(
            format!("prove circuit with 2^{} hashes", log_num_hashes).as_str(),
            |b| {
                b.iter_batched(
                    || F::rand(),
                    |init| base_circuit.generate_base_proof(init).unwrap(),
                    BatchSize::PerIteration,
                )
            },
        );

        let proof = base_circuit.generate_base_proof(F::rand()).unwrap();

        pretty_print!("proof size: {}", proof.to_bytes().len());

        group.bench_function(
            format!("verify circuit with 2^{} hashes", log_num_hashes).as_str(),
            |b| {
                b.iter_batched(
                    || (base_circuit.get_circuit_data(), proof.clone()),
                    |(data, proof)| data.verify(proof).unwrap(),
                    BatchSize::PerIteration,
                )
            },
        );
    }

    group.finish();
}

fn benchmark(c: &mut Criterion) {
    const D: usize = 2;
    type F = GoldilocksField;
    // bench_base_proof::<F, D, PoseidonGoldilocksConfig, PoseidonHash>(
    //     c,
    //     CircuitConfig::standard_recursion_config(),
    // );
    bench_base_proof::<F, D, Poseidon2GoldilocksConfig, Poseidon2Hash>(
        c,
        CircuitConfig::standard_recursion_config(),
    );
    // bench_base_proof::<F, D, PoseidonGoldilocksConfig, Poseidon2Hash>(
    //     c,
    //     CircuitConfig::standard_recursion_config(),
    // );
}

criterion_group!(name = benches;
    config = Criterion::default().sample_size(10);
    targets = benchmark);
criterion_main!(benches);
