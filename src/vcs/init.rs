use super::error::VcsResult;
use std::fs;

pub fn init() -> VcsResult<()> {
    fs::create_dir_all(".rust-vcs/commits")
        .and_then(|_| fs::write(".rust-vcs/index", ""))
        .and_then(|_| fs::write(".rust-vcs/current", ""))?;

    Ok(())
}
