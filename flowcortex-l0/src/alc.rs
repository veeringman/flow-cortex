use rand::Rng;

use sha2::{Digest, Sha256};

#[derive(Clone, Debug)]
pub struct Commitment(pub Vec<u8>);

#[derive(Clone, Debug)]
pub struct Blinding(pub u64);

pub fn commit(value: i64) -> (Commitment, Blinding) {
    //let blinding: u64 = random();
    let blinding = rand::rng().random::<u64>();
    let mut hasher = Sha256::new();
    hasher.update(value.to_le_bytes());
    hasher.update(blinding.to_le_bytes());
    let hash = hasher.finalize().to_vec();
    (Commitment(hash), Blinding(blinding))
}

pub fn verify(commitment: &Commitment, value: i64, blinding: &Blinding) -> bool {
    let mut hasher = Sha256::new();
    hasher.update(value.to_le_bytes());
    hasher.update(blinding.0.to_le_bytes());
    commitment.0 == hasher.finalize().to_vec()
}
