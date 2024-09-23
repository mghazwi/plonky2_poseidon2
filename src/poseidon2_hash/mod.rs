pub mod poseidon2;
pub mod poseidon2_goldilocks;

use plonky2::field::types::{Field, PrimeField64, Sample};
use plonky2::hash::poseidon::Poseidon;
use crate::poseidon2_hash::poseidon2::Poseidon2;

//richfield with poseidon2 added
// pub trait RichField: PrimeField64 + Poseidon + Poseidon2 {}