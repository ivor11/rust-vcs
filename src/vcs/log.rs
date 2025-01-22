use super::error::{VCSError, VCSResult};
use pager::Pager;
use std::fs;

pub fn log() -> VCSResult<()> {
    fs::exists(".rust-vcs/index").map(|x| {
        if !x {
            Err(VCSError::Uninitialized)
        } else {
            Ok(())
        }
    })??;

    let content = fs::read_to_string(".rust-vcs/index")?;

    Pager::with_pager("less -FX").setup();
    println!("{}", content);

    Ok(())
}
