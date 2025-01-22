use super::error::VCSResult;
use std::fs;

pub fn init() -> VCSResult<()> {
    fs::create_dir_all(".rust-vcs/commits")
        .and_then(|_| fs::write(".rust-vcs/index", ""))
        .and_then(|_| fs::write(".rust-vcs/current", ""))?;

    Ok(())
}
