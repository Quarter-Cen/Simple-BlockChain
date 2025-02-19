use crate::core::Blockchain;
use crate::models::Transaction;
use ed25519_dalek::Keypair;
use rand::rngs::OsRng;
use std::io::{self, Write};
use std::path::Path;
use std::sync::Arc;

// CLI manager
pub struct BlockchainCLI {
    blockchain: Blockchain,
    current_user: Option<String>,
    blockchain_file: String,
    account_file: String,
}

impl BlockchainCLI {
    pub fn new(blockchain_file: &str, accounts_file: &str) -> Self {
        let blockchain = if Path::new(blockchain_file).exists() {
            match Blockchain::load_from_file(blockchain_file, accounts_file) {
                Ok(chain) => {
                    println!("Loaded existing blockchain with {} blocks", chain.chain.len());
                    chain
                },
                Err(e) => {
                    println!("Error loading blockchain: {}. Creating new one.", e);
                    let mut csprng = OsRng;
                    let admin_keypair = Keypair::generate(&mut csprng);
                    let admin_address = hex::encode(admin_keypair.public.as_bytes());
                    let mut chain = Blockchain::new(&admin_address);
                    // Store public key first
                    chain.public_keys.insert(admin_address.clone(), admin_keypair.public);
                    // Then move the keypair
                    chain.keypairs.insert(admin_address.clone(), Arc::new(admin_keypair));
                    chain.validators.insert(admin_address.clone(), true);
                    chain
                }
            }
        } else {
            println!("Creating new blockchain...");
            let mut csprng = OsRng;
            let admin_keypair = Keypair::generate(&mut csprng);
            let admin_address = hex::encode(admin_keypair.public.as_bytes());
            let mut chain = Blockchain::new(&admin_address);
            // Store public key first
            chain.public_keys.insert(admin_address.clone(), admin_keypair.public);
            // Then move the keypair
            chain.keypairs.insert(admin_address.clone(), Arc::new(admin_keypair));
            chain.validators.insert(admin_address.clone(), true);
            println!("Created admin account: {}", admin_address);
            chain
        };

        BlockchainCLI {
            blockchain,
            current_user: None,
            blockchain_file: blockchain_file.to_string(),
            account_file: accounts_file.to_string(),
        }
    }
    
    pub fn save_blockchain(&self) -> Result<(), String> {
        self.blockchain.save_to_file(&self.blockchain_file, &self.account_file)
    }
    
    pub fn create_new_account(&mut self) -> String {
        let mut csprng = OsRng;
        let keypair = Keypair::generate(&mut csprng);
        let address = self.blockchain.register_keypair(keypair);
        address
    }
    
    pub fn select_account(&mut self, address: &str) -> Result<(), String> {
        if !self.blockchain.keypairs.contains_key(address) {
            return Err(format!("Account {} not found", address));
        }
        self.current_user = Some(address.to_string());
        Ok(())
    }
    
    pub fn get_current_user(&self) -> Result<String, &'static str> {
        match &self.current_user {
            Some(address) => Ok(address.clone()),
            None => Err("No account selected"),
        }
    }
    
    pub fn list_accounts(&self) -> Vec<String> {
        self.blockchain.keypairs.keys().cloned().collect()
    }
    
    pub fn create_transaction(&mut self, recipient: &str, amount: f64) -> Result<(), String> {
        let sender = self.get_current_user()?;
    
        if !self.blockchain.accounts.contains_key(recipient) {
            return Err(format!("Recipient {} not found", recipient));
        }
    
        let transaction = Transaction::new(sender.clone(), recipient.to_string(), amount);
    
        let keypair = self.blockchain.keypairs.get(&sender)
            .cloned() // Now possible since it's an Arc<Keypair>
            .ok_or_else(|| "Keypair not found for sender".to_string())?;
    
        self.blockchain.add_transaction(transaction, &keypair)?;
    
        Ok(())
    }
    
    pub fn create_new_block(&mut self) -> Result<(), String> {
        let validator = self.get_current_user()?;
        
        if !self.blockchain.is_validator(&validator) {
            return Err("Current account is not a validator".to_string());
        }
        
        self.blockchain.create_block(&validator)?;
        self.save_blockchain()?;
        Ok(())
    }
    
    pub fn promote_to_validator(&mut self, address: &str) -> Result<(), String> {
        let current_user = self.get_current_user()?;
        
        // Check if current user is a validator (only validators can promote)
        if !self.blockchain.is_validator(&current_user) {
            return Err("Only validators can promote accounts".to_string());
        }
        
        self.blockchain.add_validator(address.to_string())?;
        self.save_blockchain()?;
        Ok(())
    }
    
    pub fn print_balance(&self) -> Result<(), String> {
        let address = self.get_current_user()?;
        let balance = self.blockchain.get_account_balance(&address);
        println!("Balance for {}: {:.2}", address, balance);
        Ok(())
    }
    
    pub fn print_pending_transactions(&self) {
        println!("Pending Transactions: {}", self.blockchain.pending_transactions.len());
        for (i, tx) in self.blockchain.pending_transactions.iter().enumerate() {
            println!("Transaction #{}", i + 1);
            println!("{}", tx);
            println!("--------------------");
        }
    }
    
    pub fn print_blockchain_status(&self) {
        println!("Blockchain Status");
        println!("----------------");
        println!("Blocks: {}", self.blockchain.chain.len());
        println!("Accounts: {}", self.blockchain.accounts.len());
        println!("Validators: {}", self.blockchain.validators.len());
        println!("Pending Transactions: {}", self.blockchain.pending_transactions.len());
        
        let is_valid = self.blockchain.validate_chain();
        println!("Chain Validity: {}", if is_valid { "Valid" } else { "INVALID" });
        
        println!("\nLatest Block:");
        println!("{}", self.blockchain.get_latest_block());
    }
    
    pub fn run(&mut self) {
        println!("Welcome to Private Blockchain CLI");
        println!("--------------------------------");
        
        loop {
            // Display current status
            if let Some(address) = &self.current_user {
                let balance = self.blockchain.get_account_balance(address);
                let is_validator = self.blockchain.is_validator(address);
                println!("\nCurrent Account: {} (Balance: {:.2}) [{}]", 
                    address, 
                    balance,
                    if is_validator { "Validator" } else { "User" }
                );
            } else {
                println!("\nNo account selected");
            }
            
            println!("\nOptions:");
            println!("1. Create new account");
            println!("2. Select account");
            println!("3. List all accounts");
            println!("4. Check balance");
            println!("5. Create transaction");
            println!("6. View pending transactions");
            println!("7. Create new block (validators only)");
            println!("8. Promote account to validator");
            println!("9. Blockchain status");
            println!("0. Exit");
            
            print!("Enter your choice: ");
            io::stdout().flush().unwrap();
            
            let mut choice = String::new();
            io::stdin().read_line(&mut choice).unwrap();
            
            match choice.trim() {
                "1" => {
                    let address = self.create_new_account();
                    println!("Created new account: {}", address);
                    self.current_user = Some(address);
                    self.save_blockchain().unwrap_or_else(|e| println!("Error saving: {}", e));
                },
                "2" => {
                    let accounts = self.list_accounts();
                    if accounts.is_empty() {
                        println!("No accounts available. Create one first.");
                        continue;
                    }
                    
                    println!("Available accounts:");
                    for (i, account) in accounts.iter().enumerate() {
                        let balance = self.blockchain.get_account_balance(account);
                        let is_validator = self.blockchain.is_validator(account);
                        println!("{}. {} (Balance: {:.2}) [{}]", 
                            i + 1, 
                            account, 
                            balance,
                            if is_validator { "Validator" } else { "User" }
                        );
                    }
                    
                    print!("Select account number: ");
                    io::stdout().flush().unwrap();
                    
                    let mut selection = String::new();
                    io::stdin().read_line(&mut selection).unwrap();
                    
                    if let Ok(index) = selection.trim().parse::<usize>() {
                        if index > 0 && index <= accounts.len() {
                            self.select_account(&accounts[index - 1]).unwrap();
                        } else {
                            println!("Invalid selection");
                        }
                    } else {
                        println!("Invalid input");
                    }
                },
                "3" => {
                    println!("All accounts:");
                    for (i, account) in self.list_accounts().iter().enumerate() {
                        let balance = self.blockchain.get_account_balance(account);
                        let is_validator = self.blockchain.is_validator(account);
                        println!("{}. {} (Balance: {:.2}) [{}]", 
                            i + 1, 
                            account, 
                            balance,
                            if is_validator { "Validator" } else { "User" }
                        );
                    }
                },
                "4" => {
                    self.print_balance().unwrap_or_else(|e| println!("Error: {}", e));
                },
                "5" => {
                    if self.current_user.is_none() {
                        println!("No account selected. Please select an account first.");
                        continue;
                    }
                    
                    println!("Available recipients:");
                    let accounts = self.list_accounts();
                    for (i, account) in accounts.iter().enumerate() {
                        println!("{}. {}", i + 1, account);
                    }
                    
                    print!("Select recipient number: ");
                    io::stdout().flush().unwrap();
                    
                    let mut recipient_input = String::new();
                    io::stdin().read_line(&mut recipient_input).unwrap();
                    
                    let recipient_index = match recipient_input.trim().parse::<usize>() {
                        Ok(index) if index > 0 && index <= accounts.len() => index - 1,
                        _ => {
                            println!("Invalid selection");
                            continue;
                        }
                    };
                    
                    print!("Enter amount: ");
                    io::stdout().flush().unwrap();
                    
                    let mut amount_input = String::new();
                    io::stdin().read_line(&mut amount_input).unwrap();
                    
                    let amount = match amount_input.trim().parse::<f64>() {
                        Ok(amt) if amt > 0.0 => amt,
                        _ => {
                            println!("Invalid amount");
                            continue;
                        }
                    };
                    
                    match self.create_transaction(&accounts[recipient_index], amount) {
                        Ok(_) => {
                            println!("Transaction created successfully");
                            self.save_blockchain().unwrap_or_else(|e| println!("Error saving: {}", e));
                        },
                        Err(e) => println!("Error creating transaction: {}", e),
                    }
                },
                "6" => {
                    self.print_pending_transactions();
                },
                "7" => {
                    match self.create_new_block() {
                        Ok(_) => println!("Block created successfully"),
                        Err(e) => println!("Error creating block: {}", e),
                    }
                },
                "8" => {
                    if self.current_user.is_none() {
                        println!("No account selected. Please select an account first.");
                        continue;
                    }
                    
                    println!("Available accounts:");
                    let accounts = self.list_accounts();
                    for (i, account) in accounts.iter().enumerate() {
                        let is_validator = self.blockchain.is_validator(account);
                        println!("{}. {} [{}]", 
                            i + 1, 
                            account,
                            if is_validator { "Already Validator" } else { "User" }
                        );
                    }
                    
                    print!("Select account to promote: ");
                    io::stdout().flush().unwrap();
                    
                    let mut selection = String::new();
                    io::stdin().read_line(&mut selection).unwrap();
                    
                    if let Ok(index) = selection.trim().parse::<usize>() {
                        if index > 0 && index <= accounts.len() {
                            match self.promote_to_validator(&accounts[index - 1]) {
                                Ok(_) => println!("Account promoted to validator"),
                                Err(e) => println!("Error: {}", e),
                            }
                        } else {
                            println!("Invalid selection");
                        }
                    } else {
                        println!("Invalid input");
                    }
                },
                "9" => {
                    self.print_blockchain_status();
                },
                "0" => {
                    println!("Exiting...");
                    self.save_blockchain().unwrap_or_else(|e| println!("Error saving: {}", e));
                    break;
                },
                _ => println!("Invalid choice"),
            }
        }
    }
}