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

    #[command(about = "Update variable value")]
    Set {
        key: String,
        value: String,

        /// Optional description
        #[arg(short, long)]
        descriptions: Vec<String>,
    },

    #[command(about = "Delete variable")]
    Delete { key: String },

    #[command(about = "Disable/comment variable")]
    Disable { key: String },

    #[command(about = "Enable/uncomment variable")]
    Enable { key: String },
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let mut env = environment::Environment::new(&args.path);

    if let Err(e) = env.read_buf() {
        eprintln!("Failed to read buffer: {}", e);
    }

    match args.command {
        Commands::Print => {
            println!("{env}");
        }
        Commands::Get { key } => match env.get_with_key(&key) {
            Ok(val) => println!("Value: {val}"),
            Err(e) => eprintln!("Error: {}", e),
        },
        Commands::Set {
            key,
            value,
            descriptions,
        } => {
            if let Err(e) = env.set(&key, value.as_bytes().to_vec(), &descriptions) {
                eprintln!("Failed to update key: {}", e);
            } else {
                println!("Key updated successfully");
            }
        }
        Commands::Delete { key } => {
            if let Err(e) = env.delete(&key) {
                eprintln!("Failed to delete key: {}", e);
            } else {
                println!("Key deleted successfully");
            }
        }
        Commands::Disable { key } => {
            if let Err(e) = env.toggle(&key, environment::KeyStatus::Disable) {
                eprintln!("Failed toggle update key: {}", e);
            } else {
                println!("Key disabled successfully");
            }
        }
        Commands::Enable { key } => {
            if let Err(e) = env.toggle(&key, environment::KeyStatus::Enable) {
                eprintln!("Failed toggle update key: {}", e);
            } else {
                println!("Key enabled successfully");
            }
        }
    }

    Ok(())
}
