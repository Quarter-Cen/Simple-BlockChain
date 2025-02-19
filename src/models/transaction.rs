use ed25519_dalek::{Keypair, PublicKey, Signature, Signer, Verifier};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fmt;
use std::time::{SystemTime, UNIX_EPOCH};
use std::fs;
use hex;

#[derive(Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub sender: String,
    pub recipient: String,
    pub amount: f64,
    pub signature: Option<String>,
    pub timestamp: u64,
}

impl Transaction {
    pub fn new(sender: String, recipient: String, amount: f64) -> Self {
        Transaction {
            sender,
            recipient,
            amount,
            signature: None,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    pub fn calculate_hash(&self) -> String {
        let transaction_data = format!(
            "{}{}{}{}",
            self.sender, self.recipient, self.amount, self.timestamp
        );
        let mut hasher = Sha256::new();
        hasher.update(transaction_data.as_bytes());
        format!("{:x}", hasher.finalize())
    }

    pub fn sign(&mut self, keypair: &Keypair) -> Result<(), &'static str> {
        if self.sender == "0" {
            return Err("Cannot sign genesis transaction");
        }

        let transaction_hash = self.calculate_hash();
        let signature = keypair.sign(transaction_hash.as_bytes());
        self.signature = Some(hex::encode(signature.to_bytes()));
        Ok(())
    }

    fn load_accounts() -> HashMap<String, String> {
        let file_content = fs::read_to_string("accounts.json")
            .expect("Unable to read file");

        let accounts: HashMap<String, String> = serde_json::from_str(&file_content)
            .expect("Unable to parse JSON");

        accounts
    }

    pub fn is_valid(&self) -> bool {
        // Genesis transactions are always valid
        if self.sender == "0" {
            println!("Transaction is a genesis transaction, always valid.");
            return true;
        }

        // Must have a signature
        let signature = match &self.signature {
            Some(sig) => {
                sig
            },
            None => {
                println!("Transaction has no signature.");
                return false;
            },
        };

        // Load public keys from the accounts file
        let accounts = Self::load_accounts();

        // Look up the sender's public key
        let public_key_data = match accounts.get(&self.sender) {
            Some(data) => data,
            None => {
                println!("Sender not found in accounts.");
                return false;
            },
        };

        // Extract the public key part (assuming the format is "public_key:secret_key")
        let parts: Vec<&str> = public_key_data.split(':').collect();
        if parts.len() != 2 {
            println!("Invalid public key data format.");
            return false;
        }

        let public_key_str = parts[1];

        // Convert the public key from string to ed25519 PublicKey
        let public_key = match PublicKey::from_bytes(&hex::decode(public_key_str).unwrap()) {
            Ok(pk) => pk,
            Err(_) => {
                println!("Failed to parse public key.");
                return false;
            },
        };

        // Signature must be valid hex
        let signature_bytes = match hex::decode(signature) {
            Ok(bytes) => bytes,
            Err(_) => {
                println!("Failed to decode signature as hex.");
                return false;
            },
        };

        // Signature must be valid ed25519
        let signature = match Signature::from_bytes(&signature_bytes) {
            Ok(sig) => sig,
            Err(_) => {
                println!("Failed to convert signature bytes into a valid ed25519 signature.");
                return false;
            },
        };
        
        // Calculate transaction hash
        let transaction_hash = self.calculate_hash();

        let result = public_key.verify(transaction_hash.as_bytes(), &signature);
        if result.is_ok() {
            println!("Signature is valid.");
        } else {
            println!("Signature verification failed.");
        }

        // Return the result of the verification
        result.is_ok()

    }

}

impl fmt::Display for Transaction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "From: {}\nTo: {}\nAmount: {:.2}\nTimestamp: {}\nSigned: {}",
            self.sender,
            self.recipient,
            self.amount,
            self.timestamp,
            self.signature.is_some()
        )
    }
}
