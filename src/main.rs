use clap::{Parser, Subcommand};
use std::io::ErrorKind;

mod commit;
mod init;
mod status;

/// Simple Version Control System
#[derive(Parser, Debug)]
#[command(name = "rust-vcs", version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    Init,
    Status,
    Add {
        /// Paths to add to staging
        path: Vec<String>,
    },
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,
    },
    Checkout {
        /// Commit to checkout to
        commit: String,
    },
}

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Init => match init::init() {
            Ok(_) => println!("Initialzed VCS!"),
            Err(err) => match err.kind() {
                ErrorKind::AlreadyExists => eprint!("VCS already initialized!"),
                _ => eprint!("Failed to initialize VCS"),
            },
        },
        Command::Commit { message } => {
            print!("Commit with message {}", message);
            let _ = commit::commit();
        }
        Command::Add { path } => {
            for p in path {
                println!("{}", p);
            }
        }
        Command::Checkout { .. } => todo!(),
        Command::Status => status::status().unwrap_or_else(|err| println!("{}", err)),
    }
}
