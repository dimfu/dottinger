mod environment;

use clap::{Parser, Subcommand};
use std::io;

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    /// Optional path to the env file, default to current directory's .env
    #[arg(short, long, default_value_t = String::from(".env"))]
    path: String,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    #[command(about = "Print all environment variables")]
    Print,
    #[command(about = "Get environment value by providing key")]
    Get { key: String },
    #[command(about = "Update existing environment variable")]
    Set { key: String, value: String },
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut env = environment::Environment::new(&args.path);

    if let Err(e) = env.read_buf() {
        eprintln!("Failed to read buffer: {}", e);
    }

    match args.command {
        Commands::Print => {
            for (key, value) in env.map.iter() {
                println!("{key}:{}", String::from_utf8_lossy(value));
            }
        }
        Commands::Get { key } => match env.get_with_key(&key) {
            Ok(val) => println!("Value: {val}"),
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Set { key, value } => {
            if let Err(e) = env.set(&key, value.as_bytes().to_vec()) {
                eprintln!("Failed to update key: {}", e);
            } else {
                println!("Key updated successfully");
            }
        }
    }

    Ok(())
}
