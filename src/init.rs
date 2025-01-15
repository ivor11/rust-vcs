use std::fs;
use std::io::Error;

// struct Repository {}

pub fn init() -> Result<(), Error> {
    fs::create_dir_all(".rust-vcs/commits")
        .and_then(|_| fs::write(".rust-vcs/index", ""))
        .and_then(|_| fs::write(".rust-vcs/current", ""))
}
