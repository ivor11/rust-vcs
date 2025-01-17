use crate::status::{self, VCSTree};
use std::fs;
use std::io::{Error, ErrorKind};
use std::time::SystemTime;

pub fn commit() -> Result<(), Error> {
    let tree = status::get_tree_structure(".".into())?;
    let commit_id = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Failed to get system time")
        .as_secs();

    let commit_root = format!(".rust-vcs/commits/{}", commit_id);
    fs::create_dir_all(format!("{}/meta", commit_root))
        .and_then(|_| fs::create_dir_all(format!("{}/data", commit_root)))?;

    let json_data = serde_json::to_string(&tree.copy_to(format!("{}/data", commit_root))).unwrap();
    fs::write(format!("{}/meta/tree.json", commit_root), json_data)?;

    fs::write(".rust-vcs/current", commit_id.to_string())?;

    Ok(())
}
