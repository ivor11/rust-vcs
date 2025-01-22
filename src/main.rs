use clap::Parser;
use rust_vcs::cli::args::{Args, Command};
use rust_vcs::vcs::{checkout, commit, init, log, status};

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Init => match init::init() {
            Ok(_) => println!("Initialzed VCS!"),
            Err(err) => eprintln!("Failed to initialize VCS: {}", err),
        },
        Command::Commit { message } => {
            print!("Commit with message {}", message);
            if let Err(err) = commit::commit(message) {
                eprintln!("Failed to commit: {}", err);
            }
        }
        Command::Add { path } => {
            for p in path {
                println!("{}", p);
            }
        }
        Command::Checkout { commit } => {
            if let Err(err) = checkout::checkout(commit.clone()) {
                eprintln!("Failed to checkout: {}", err);
            } else {
                println!("Checkout to commit {}", commit);
            }
        }
        Command::Log { .. } => {
            log::log().unwrap_or_else(|err| eprintln!("Error displaying log: {}", err))
        }
        Command::Status => {
            status::status().unwrap_or_else(|err| eprintln!("Error fetching status: {}", err))
        }
    }
}
