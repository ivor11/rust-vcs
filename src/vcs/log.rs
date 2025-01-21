use std::fs;
use pager::Pager;
use super::error::VcsResult;

pub fn log() -> VcsResult<()> {
    let content = fs::read_to_string(".rust-vcs/index")?;

   Pager::with_pager("less -FX").setup();
   println!("{}", content);

    Ok(())
}