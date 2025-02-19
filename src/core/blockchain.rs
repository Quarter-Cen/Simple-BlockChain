use crate::models::{Block, Transaction};
use ed25519_dalek::{Keypair, PublicKey, SecretKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

/// The main blockchain data structure
#[derive(Serialize, Deserialize)]
pub struct Blockchain {
    pub chain: Vec<Block>,
    pub pending_transactions: Vec<Transaction>,
    pub accounts: HashMap<String, f64>,
    #[serde(skip)]
    pub public_keys: HashMap<String, PublicKey>,
    pub validators: HashMap<String, bool>,
    #[serde(skip)]
    pub keypairs: HashMap<String, Arc<Keypair>>,
}

impl Blockchain {
    /// Creates a new blockchain with an initial balance for the genesis address
    pub fn new(genesis_address: &str) -> Self {
        let mut blockchain = Blockchain {
            chain: Vec::new(),
            pending_transactions: Vec::new(),
            accounts: HashMap::new(),
            public_keys: HashMap::new(),
            validators: HashMap::new(),
            keypairs: HashMap::new(),
        };

        // Create genesis transaction
        let genesis_transaction = Transaction::new(
            "0".to_string(),
            genesis_address.to_string(),
            1000.0,
        );
        
        blockchain.pending_transactions.push(genesis_transaction);
        blockchain.accounts.insert(genesis_address.to_string(), 1000.0);
        blockchain.create_genesis_block(genesis_address);
        blockchain
    }

    /// Creates the genesis (first) block in the chain
    pub fn create_genesis_block(&mut self, genesis_address: &str) {
        let genesis_block = Block::new(
            0, 
            self.pending_transactions.clone(), 
            "0".to_string(),
            genesis_address.to_string()
        );
        self.chain.push(genesis_block);
        self.pending_transactions.clear();
    }

    /// Returns a reference to the most recent block
    pub fn get_latest_block(&self) -> &Block {
        self.chain.last().expect("Chain should not be empty")
    }

    /// Adds a transaction to the pending transactions pool
    pub fn add_transaction(&mut self, mut transaction: Transaction, keypair: &Keypair) -> Result<(), String> {
        // Check if sender has enough balance (except for genesis transactions)
        if transaction.sender != "0" {
            let sender_balance = self.accounts.get(&transaction.sender).unwrap_or(&0.0);
            if *sender_balance < transaction.amount {
                return Err("Insufficient balance for transaction".to_string());
            }
            
            // Sign the transaction
            transaction.sign(keypair).map_err(|e| e.to_string())?;
        }

        // Add to pending transactions
        self.pending_transactions.push(transaction);
        Ok(())
    }

    /// Registers a keypair with the blockchain and returns the associated address
    pub fn register_keypair(&mut self, keypair: Keypair) -> String {
        let address = hex::encode(keypair.public.as_bytes());
        
        // Store public key
        self.public_keys.insert(address.clone(), keypair.public);
        
        // Store keypair
        self.keypairs.insert(address.clone(), Arc::new(keypair));
        
        // Initialize account balance to zero
        self.accounts.entry(address.clone()).or_insert(0.0);
        
        address
    }

    /// Adds an account to the validator set
    pub fn add_validator(&mut self, address: String) -> Result<(), String> {
        if !self.public_keys.contains_key(&address) {
            return Err(format!("Address {} is not registered", address));
        }
        self.validators.insert(address, true);
        Ok(())
    }

    /// Checks if an address is a validator
    pub fn is_validator(&self, address: &str) -> bool {
        *self.validators.get(address).unwrap_or(&false)
    }

    /// Creates a new block containing the pending transactions
    pub fn create_block(&mut self, validator_address: &str) -> Result<Block, String> {
        // Ensure validator authorization
        if !self.is_validator(validator_address) {
            return Err("Only authorized validators can create blocks".to_string());
        }

        // Ensure there are transactions to include
        if self.pending_transactions.is_empty() {
            return Err("No pending transactions to include in block".to_string());
        }

        // Create new block
        let block = Block::new(
            self.chain.len() as u32,
            self.pending_transactions.clone(),
            self.get_latest_block().hash.clone(),
            validator_address.to_string(),
        );

        // Update chain
        self.chain.push(block.clone());

        // Update account balances
        self.apply_transactions();

        // Clear pending transactions
        self.pending_transactions.clear();

        Ok(block)
    }

    /// Applies all pending transactions to account balances
    fn apply_transactions(&mut self) {
        for tx in &self.pending_transactions {
            // Debit sender (except genesis)
            if tx.sender != "0" {
                *self.accounts.entry(tx.sender.clone()).or_insert(0.0) -= tx.amount;
            }
            
            // Credit recipient
            *self.accounts.entry(tx.recipient.clone()).or_insert(0.0) += tx.amount;
        }
    }

    /// Validates the entire blockchain
    pub fn validate_chain(&self) -> bool {
        // Empty chain is valid
        if self.chain.is_empty() {
            return true;
        }
        
        // Validate each block starting from the second one
        for i in 1..self.chain.len() {
            let current_block = &self.chain[i];
            let previous_block = &self.chain[i - 1];

            // Check hash integrity
            if current_block.hash != current_block.calculate_hash() {
                print!("hash integrity failed");
                return false;
            }

            // Check previous hash link
            if current_block.previous_hash != previous_block.hash {
                print!("previous hash link failed");
                return false;
            }

            // Validate all transactions in the block
            for tx in &current_block.transactions {
                if !tx.is_valid() {
                    print!("Validate all transactions in the block failed :");
                    return false;
                }
            }

            // Check if the block was created by a valid validator
            if !self.validators.get(&current_block.validator).unwrap_or(&false) {
                print!("created by a valid validator failed");
                return false;
            }
        }

        true
    }

    /// Gets the balance for an account
    pub fn get_account_balance(&self, address: &str) -> f64 {
        *self.accounts.get(address).unwrap_or(&0.0)
    }
    
    /// Saves the blockchain to files
    pub fn save_to_file(&self, filename: &str, accounts_file: &str) -> Result<(), String> {
        let blockchain_json = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize: {}", e))?;
    
        fs::write(filename, blockchain_json)
            .map_err(|e| format!("Failed to write file: {}", e))?;
    
    // การบันทึก keypair (public + secret key ทั้งหมดในรูปแบบ hex)
        let accounts_json: HashMap<String, String> = self.keypairs.iter()
            .map(|(address, keypair)| {
                let secret_hex = hex::encode(keypair.secret.as_bytes()); // แปลง secret key เป็น hex
                let public_hex = hex::encode(keypair.public.as_bytes()); // แปลง public key เป็น hex
                (address.clone(), format!("{}:{}", secret_hex, public_hex)) // เก็บไว้ในรูปแบบ secret:public
            })
            .collect();
        
        // Serialize the accounts data into a pretty JSON format
        let pretty_json = serde_json::to_string_pretty(&accounts_json)
            .map_err(|e| format!("Failed to serialize accounts: {}", e))?;

        // Write the pretty JSON to the accounts file
        fs::write(accounts_file, pretty_json)
            .map_err(|e| format!("Unable to write accounts to file: {}", e))?;

        Ok(())
    }
    
    
    pub fn load_from_file(blockchain_file: &str, accounts_file: &str) -> Result<Self, String> {
        let blockchain_data = fs::read_to_string(blockchain_file)
            .map_err(|_| format!("Failed to read blockchain file: {}", blockchain_file))?;
        let mut blockchain: Blockchain = serde_json::from_str(&blockchain_data)
            .map_err(|_| "Failed to parse blockchain file".to_string())?;
    
        let accounts_data = fs::read_to_string(accounts_file)
            .map_err(|_| format!("Failed to read accounts file: {}", accounts_file))?;
        let accounts: HashMap<String, String> = serde_json::from_str(&accounts_data)
            .map_err(|_| "Failed to parse accounts file".to_string())?;
    
        for (address, keypair_str) in accounts {
            // แยก secret และ public key
            let parts: Vec<&str> = keypair_str.split(':').collect();
            if parts.len() != 2 {
                return Err(format!("Invalid keypair format for address: {}", address));
            }
            let secret_hex = parts[0];
            let public_hex = parts[1];
    
            // แปลง hex เป็น bytes
            let secret_bytes = hex::decode(secret_hex)
                .map_err(|_| format!("Invalid secret key hex for address: {}", address))?;
            let public_bytes = hex::decode(public_hex)
                .map_err(|_| format!("Invalid public key hex for address: {}", address))?;
    
            // สร้าง Keypair จาก bytes
            let public_key = PublicKey::from_bytes(&public_bytes)
                .map_err(|_| format!("Invalid public key for address: {}", address))?;
            let secret_key = SecretKey::from_bytes(&secret_bytes)
                .map_err(|_| format!("Invalid secret key for address: {}", address))?;
            let keypair = Keypair { public: public_key, secret: secret_key };
    
            println!("Loaded account: {}", address);
            blockchain.keypairs.insert(address.clone(), Arc::new(keypair));
        }
    
        Ok(blockchain)
    }
    
    
    
}