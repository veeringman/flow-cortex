#[derive(Clone, Debug)]
pub struct Transaction {
    pub key: Vec<u8>, // account / prefix key
    pub amount: i64,  // positive or negative delta
}

#[derive(Clone, Debug)]
pub struct Frequency {
    pub count: u64,
    pub sum: i64,
}

impl Frequency {
    pub fn new() -> Self {
        Self { count: 0, sum: 0 }
    }

    pub fn apply(&mut self, delta: i64) {
        self.count += 1;
        self.sum += delta;
    }
}

impl Default for Frequency {
    fn default() -> Self {
        Self::new()
    }
}
