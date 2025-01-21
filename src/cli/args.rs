use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "rvcs", version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    Init,
    Status,
    Log,
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
