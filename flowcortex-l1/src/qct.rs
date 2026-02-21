use crate::types::{ReadWriteSet, QCTProof};

/// Generate a dummy proof for the given read/write set. For illustration we just
/// hash the concatenated keys. Real QCT proofs would contain polynomial
/// commitments, frequency aggregates, etc.
pub fn prove(rw: &ReadWriteSet) -> QCTProof {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    for key in &rw.reads {
        hasher.update(key.as_bytes());
    }
    for key in &rw.writes {
        hasher.update(key.as_bytes());
    }
    QCTProof(hasher.finalize().to_vec())
}

/// Verify that a proof matches a read/write set. Here we simply re-run `prove`.
pub fn verify(proof: &QCTProof, rw: &ReadWriteSet) -> bool {
    let expected = prove(rw);
    &expected.0 == &proof.0
}
