use clap::Parser;
use rvcs::cli::args::{Args, Command};
use rvcs::vcs::{init, commit, status};

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Init => match init::init() {
            Ok(_) => println!("Initialzed VCS!"),
            Err(err) => eprintln!("Failed to initialize VCS: {}", err)
        },
        Command::Commit { message } => {
            print!("Commit with message {}", message);
            if let Err(err) = commit::commit() {
                eprintln!("Failed to commit: {}", err);
            }
        }
        Command::Add { path } => {
            for p in path {
                println!("{}", p);
            }
        }
        Command::Checkout { .. } => todo!(),
        Command::Status => status::status().unwrap_or_else(|err| eprintln!("Error fetching status: {}", err)),
    }
}
