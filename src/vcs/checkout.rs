use crate::config::Settings;

use super::status;
use super::{
    error::{VCSError, VCSResult},
    tree::VCSTree,
};
use std::{fs, path::PathBuf};

pub fn checkout(commit: String, config: Settings) -> VCSResult<()> {
    fs::exists(".rust-vcs/index").map(|x| {
        if !x {
            Err(VCSError::Uninitialized)
        } else {
            Ok(())
        }
    })??;

    if let Some(_) = status::get_current_diff_tree(&config)? {
        return Err(VCSError::Other("Uncommitted changes".into()));
    }

    let commit_path = PathBuf::from(format!(".rust-vcs/commits/{}/meta/tree.json", commit));

    let commit_tree: VCSTree = serde_json::from_str(&fs::read_to_string(commit_path)?)?;

    commit_tree.copy_to(PathBuf::from("."))?;

    fs::write(".rust-vcs/current", commit)?;

    Ok(())
}
