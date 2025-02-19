use crate::models::transaction::Transaction;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};

/// Represents a block in the blockchain
#[derive(Clone, Serialize, Deserialize)]
pub struct Block {
    pub index: u32,
    pub timestamp: u64,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub validator: String,
}

impl Block {
    /// Creates a new block with the given properties
    pub fn new(
        index: u32,
        transactions: Vec<Transaction>,
        previous_hash: String,
        validator: String,
    ) -> Self {
        let mut block = Block {
            index,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            transactions,
            previous_hash,
            hash: String::new(),
            validator,
        };

        block.hash = block.calculate_hash();
        block
    }

    /// Calculates the hash of this block
    pub fn calculate_hash(&self) -> String {
        let block_data = format!(
            "{}{}{}{}{}",
            self.index,
            self.timestamp,
            serde_json::to_string(&self.transactions).unwrap_or_default(),
            self.previous_hash,
            self.validator
        );
        let mut hasher = Sha256::new();
        hasher.update(block_data.as_bytes());
        format!("{:x}", hasher.finalize())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Block #{}\n  Hash: {}\n  Previous Hash: {}\n  Transactions: {}\n  Validator: {}\n  Timestamp: {}",
            self.index,
            self.hash,
            self.previous_hash,
            self.transactions.len(),
            self.validator,
            self.timestamp
        )
    }
}