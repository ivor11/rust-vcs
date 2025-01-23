use clap::Parser;
use config::Config;
use rust_vcs::cli::args::{Args, Command};
use rust_vcs::config::Settings;
use rust_vcs::vcs::{checkout, commit, init, log, status};

fn main() {
    let mut settings = Config::builder()
        .add_source(config::File::with_name("env"))
        .build()
        .unwrap()
        .try_deserialize::<Settings>()
        .unwrap();

    settings.ignore.push(".rust-vcs".to_string());

    let args = Args::parse();
    match args.command {
        Command::Init => match init::init() {
            Ok(_) => println!("Initialzed VCS!"),
            Err(err) => eprintln!("Failed to initialize VCS: {}", err),
        },
        Command::Commit { message } => {
            print!("Commit with message {}", message);
            if let Err(err) = commit::commit(message, settings) {
                eprintln!("Failed to commit: {}", err);
            }
        }
        Command::Add { path } => {
            for p in path {
                println!("{}", p);
            }
        }
        Command::Checkout { commit } => {
            if let Err(err) = checkout::checkout(commit.clone(), settings) {
                eprintln!("Failed to checkout: {}", err);
            } else {
                println!("Checkout to commit {}", commit);
            }
        }
        Command::Log { .. } => {
            log::log().unwrap_or_else(|err| eprintln!("Error displaying log: {}", err))
        }
        Command::Status => status::status(settings)
            .unwrap_or_else(|err| eprintln!("Error fetching status: {}", err)),
    }
}
