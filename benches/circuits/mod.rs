use anyhow::Result;
use plonky2::field::extension::Extendable;
use plonky2::hash::hash_types::RichField;
use plonky2::hash::hashing::hash_n_to_m_no_pad;
use plonky2::iop::target::Target;
use plonky2::iop::witness::{PartialWitness, WitnessWrite, Witness};
use plonky2::plonk::circuit_builder::CircuitBuilder;
use plonky2::plonk::circuit_data::{CircuitConfig, CircuitData, VerifierCircuitData};
use plonky2::plonk::config::{AlgebraicHasher, GenericConfig, Hasher};
use plonky2::plonk::proof::ProofWithPublicInputs;
use std::marker::PhantomData;

use plonky2::hash::merkle_tree::MerkleTree;
use plonky2::hash::poseidon::PoseidonHash;

use plonky2::hash::hash_types::{HashOutTarget, MerkleCapTarget};
use plonky2::hash::merkle_proofs::MerkleProofTarget;
use plonky2_poseidon2::poseidon2_hash::poseidon2::{Poseidon2, Poseidon2Hash};

pub struct BaseCircuit<
    F: RichField + Extendable<D> + Poseidon2,
    C: GenericConfig<D, F = F>,
    const D: usize,
    H: Hasher<F> + AlgebraicHasher<F>,
> {
    private_input: Target,
    public_input: Target,
    public_output: Target,
    circuit_data: CircuitData<F, C, D>,
    num_powers: usize,
    _hasher: PhantomData<H>,
}

impl<
    F: RichField + Extendable<D> + Poseidon2,
    C: GenericConfig<D, F = F>,
    const D: usize,
    H: Hasher<F> + AlgebraicHasher<F>,
> BaseCircuit<F, C, D, H>
{
    pub fn build_base_circuit(config: CircuitConfig, log_num_hashes: usize) -> Self {
        let num_hashes: usize = 1usize << log_num_hashes;

        let mut builder = CircuitBuilder::<F, D>::new(config);
        let mut res_t = builder.add_virtual_public_input();
        let init_t = res_t;
        let zero = builder.zero();
        let to_be_hashed_t = builder.add_virtual_target();
        for _ in 0..num_hashes {
            res_t = builder.mul(res_t, init_t);
            res_t = builder.hash_n_to_m_no_pad::<H>(vec![res_t, to_be_hashed_t, zero, zero], 1)[0];
        }

        let out_t = builder.add_virtual_public_input();
        let is_eq_t = builder.is_equal(out_t, res_t);
        builder.assert_one(is_eq_t.target);

        let data = builder.build::<C>();

        Self {
            private_input: to_be_hashed_t,
            public_input: init_t,
            public_output: out_t,
            circuit_data: data,
            num_powers: num_hashes,
            _hasher: PhantomData::<H>,
        }
    }

    pub fn generate_base_proof(&self, init: F) -> Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::<F>::new();

        pw.set_target(self.public_input, init);
        let to_be_hashed = F::rand();
        pw.set_target(self.private_input, to_be_hashed);
        let mut res = init;
        for _ in 0..self.num_powers {
            res = res.mul(init);
            res =
                hash_n_to_m_no_pad::<_, H::Permutation>(&[res, to_be_hashed, F::ZERO, F::ZERO], 1)
                    [0];
        }

        pw.set_target(self.public_output, res);

        let proof = self.circuit_data.prove(pw)?;

        self.circuit_data.verify(proof.clone())?;

        assert_eq!(proof.public_inputs[1], res);

        Ok(proof)
    }

    pub fn get_circuit_data(&self) -> &CircuitData<F, C, D> {
        &self.circuit_data
    }
}

//****** Merkle tree verification circuit with Poseidon & Poseidon 2 ******** //

pub struct BaseCircuit2<
    F: RichField + Extendable<D> + Poseidon2,
    C: GenericConfig<D, F = F>,
    const D: usize,
    H: Hasher<F> + AlgebraicHasher<F>,
> {
    private_input: Target,
    public_input: Target,
    public_output: Target,
    circuit_data: CircuitData<F, C, D>,
    num_powers: usize,
    _hasher: PhantomData<H>,
}

impl<
    F: RichField + Extendable<D> + Poseidon2,
    C: GenericConfig<D, F = F>,
    const D: usize,
    H: Hasher<F> + AlgebraicHasher<F>,
> BaseCircuit2<F, C, D, H>
{
    pub fn build_base_circuit(config: CircuitConfig, log_num_hashes: usize) -> Self {
        let num_hashes: usize = 1usize << log_num_hashes;

        let mut builder = CircuitBuilder::<F, D>::new(config);
        let mut res_t = builder.add_virtual_public_input();
        let init_t = res_t;
        let zero = builder.zero();
        let to_be_hashed_t = builder.add_virtual_target();
        for _ in 0..num_hashes {
            res_t = builder.mul(res_t, init_t);
            res_t = builder.hash_n_to_m_no_pad::<H>(vec![res_t, to_be_hashed_t, zero, zero], 1)[0];
        }

        let out_t = builder.add_virtual_public_input();
        let is_eq_t = builder.is_equal(out_t, res_t);
        builder.assert_one(is_eq_t.target);

        let data = builder.build::<C>();

        Self {
            private_input: to_be_hashed_t,
            public_input: init_t,
            public_output: out_t,
            circuit_data: data,
            num_powers: num_hashes,
            _hasher: PhantomData::<H>,
        }
    }

    pub fn generate_base_proof(&self, init: F) -> Result<ProofWithPublicInputs<F, C, D>> {
        let mut pw = PartialWitness::<F>::new();

        pw.set_target(self.public_input, init);
        let to_be_hashed = F::rand();
        pw.set_target(self.private_input, to_be_hashed);
        let mut res = init;
        for _ in 0..self.num_powers {
            res = res.mul(init);
            res =
                hash_n_to_m_no_pad::<_, H::Permutation>(&[res, to_be_hashed, F::ZERO, F::ZERO], 1)
                    [0];
        }

        pw.set_target(self.public_output, res);

        let proof = self.circuit_data.prove(pw)?;

        self.circuit_data.verify(proof.clone())?;

        assert_eq!(proof.public_inputs[1], res);

        Ok(proof)
    }

    pub fn get_circuit_data(&self) -> &CircuitData<F, C, D> {
        &self.circuit_data
    }
}

pub struct MT<
    F: RichField + Extendable<D> + Poseidon2,
    const D: usize,
    H: Hasher<F> + AlgebraicHasher<F>,
> (pub MerkleTree<F, H>);

pub struct MTTargets {
    merkle_root: HashOutTarget,
    merkle_proof: MerkleProofTarget,
    leaf_data: [Target; 4],
    leaf_index: Target,
}

impl<
    F: RichField + Extendable<D> + Poseidon2,
    const D: usize,
    H: Hasher<F> + AlgebraicHasher<F>,
> MT<F, D, H>{
    pub fn tree_height(&self) -> usize {
        self.0.leaves.len().trailing_zeros() as usize
    }

    pub fn mt_circuit(&self, builder: &mut CircuitBuilder<F, D>) -> MTTargets {
        // Register public inputs.
        let merkle_root = builder.add_virtual_hash();
        builder.register_public_inputs(&merkle_root.elements);

        // Merkle proof
        let merkle_proof = MerkleProofTarget {
            siblings: builder.add_virtual_hashes(self.tree_height()),
        };

        // Verify Merkle proof.
        let leaf_data: [Target; 4] = builder.add_virtual_targets(4).try_into().unwrap();
        let leaf_index = builder.add_virtual_target();
        let leaf_index_bits = builder.split_le(leaf_index, self.tree_height());
        let zero = builder.zero();
        builder.verify_merkle_proof::<H>(
            [leaf_data, [zero; 4]].concat(),
            &leaf_index_bits,
            merkle_root,
            &merkle_proof,
        );

        MTTargets {
            merkle_root,
            merkle_proof,
            leaf_data,
            leaf_index,
        }
    }

    pub fn fill_targets(
        &self,
        pw: &mut PartialWitness<F>,
        leaf_data: [F; 4],
        // topic: [F; 4],
        leaf_index: usize,
        targets: MTTargets,
    ) {
        let MTTargets {
            merkle_root,
            merkle_proof: merkle_proof_target,
            leaf_data: leaf_data_target,
            leaf_index: leaf_index_target,
        } = targets;

        pw.set_hash_target(merkle_root, self.0.cap.0[0]);
        for i in 0..leaf_data.len() {
            pw.set_target(leaf_data_target[i], leaf_data[i]);
        }
        pw.set_target(
            leaf_index_target,
            F::from_canonical_usize(leaf_index),
        );

        let merkle_proof = self.0.prove(leaf_index);
        for (ht, h) in merkle_proof_target
            .siblings
            .into_iter()
            .zip(merkle_proof.siblings)
        {
            pw.set_hash_target(ht, h);
        }
    }
}