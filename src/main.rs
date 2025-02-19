mod models;
mod core;
mod cli;
use cli::blockchain_cli::BlockchainCLI;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    let blockchain_file = if args.len() > 1 {
        &args[1]
    } else {
        "blockchain.json"
    };

    let account_file = if args.len() > 2 {
        &args[2]
    } else {
        "accounts.json"
    };

    let mut cli = BlockchainCLI::new(blockchain_file, account_file);
    cli.run();
    
    Ok(())
}