mod models;
mod core;
mod cli;

use cli::blockchain_cli::BlockchainCLI;
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();

    let default_blockchain_file = "blockchain.json".to_string();
    let blockchain_file = args.get(1).unwrap_or(&default_blockchain_file);

    let default_account_file = "accounts.json".to_string();
    let account_file = args.get(2).unwrap_or(&default_account_file);

    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

    println!("Starting blockchain node on port: {}", port);
    
    let mut cli = BlockchainCLI::new(blockchain_file, account_file);
    cli.run();
    
    Ok(())
}
