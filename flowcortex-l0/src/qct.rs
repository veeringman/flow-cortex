use crate::alc::{commit, Blinding, Commitment};
use crate::types::{Frequency, Transaction};
use std::collections::HashMap;

#[derive(Clone)]
pub struct QCTNode {
    pub prefix: Vec<u8>,
    pub commitment: Commitment,
    pub blinding: Blinding,
    pub frequency: Frequency,
    pub children: HashMap<u8, QCTNode>,
}

impl QCTNode {
    pub fn new(prefix: Vec<u8>) -> Self {
        let (c, b) = commit(0);
        Self {
            prefix,
            commitment: c,
            blinding: b,
            frequency: Frequency::new(),
            children: HashMap::new(),
        }
    }

    pub fn insert(&mut self, tx: &Transaction, depth: usize) {
        self.frequency.apply(tx.amount);

        let (c, b) = commit(self.frequency.sum);
        self.commitment = c;
        self.blinding = b;

        if depth >= tx.key.len() {
            return;
        }

        let next = tx.key[depth];
        self.children
            .entry(next)
            .or_insert_with(|| QCTNode::new(self.prefix.clone()))
            .insert(tx, depth + 1);
    }
}
