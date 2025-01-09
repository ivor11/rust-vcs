use clap::{Parser, Subcommand};

/// Simple VCS system
#[derive(Parser, Debug)]
#[command(name = "rust-vcs", version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, Debug, Subcommand)]
enum Command {
    Init,
    Commit {        
        /// Commit message
        #[arg(short, long)]
        message: String,
    }
}

fn main() {
    let args = Args::parse();
    match args.command {
        Command::Init => println!("init the repo"),
        Command::Commit { message } => print!("Commit with message {}", message)
    }
}