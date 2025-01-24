use crate::config::Settings;

use super::error::{VCSError, VCSResult};
use super::tree::{VCSDirectory, VCSFile, VCSKind, VCSTree};
use super::util;
use clap::error::Result;
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::PathBuf;

pub fn status(config: Settings) -> VCSResult<()> {
    util::check_vcs_initialized()?;

    let current_commit = fs::read_to_string(".rust-vcs/current")?;
    let diff = get_current_diff_tree(&config)?;

    if current_commit.is_empty() {
        //new instance
        println!("New VCS Repository: Untracked files");
    };
    match diff {
        None => println!("No changes to commit"),
        Some(t) => {
            println!("Changes:");
            print!("{}", t.to_string());
        }
    }

    Ok(())
}

pub fn get_current_diff_tree(config: &Settings) -> VCSResult<Option<VCSTree>> {
    let current_commit = fs::read_to_string(".rust-vcs/current")?;

    let matched_commit = fs::read_dir(".rust-vcs/commits")?
        .filter(|dir| {
            dir.as_ref().expect("match fail").file_name().into_string()
                == Ok(current_commit.to_owned())
        })
        .last();

    let tree: VCSTree = get_tree_structure(".".into(), config)?;

    Ok(match matched_commit {
        None => Some(tree),
        Some(matched_value) => {
            let matched_commit = matched_value?;
            let commit_id = matched_commit.file_name();
            let mut path = PathBuf::from("./.rust-vcs/commits");
            path.push(commit_id);

            let mut old_tree_path = path.clone();
            old_tree_path.push("meta");
            old_tree_path.push("tree.json");

            let old_tree: VCSTree = serde_json::from_str(&fs::read_to_string(old_tree_path)?)?;

            tree.diff_tree(old_tree)
        }
    })
}

pub fn get_tree_structure(root: PathBuf, config: &Settings) -> VCSResult<VCSTree> {
    let ignore = &config.ignore;
    let dir_contents = fs::read_dir(&root)?
        .filter(|res| {
            res.as_ref()
                .map(|entry| {
                    !ignore.contains(
                        &entry
                            .file_name()
                            .to_os_string()
                            .into_string()
                            .unwrap_or("".into()),
                    )
                })
                .expect("FAILED TO FILTER")
        })
        .map(|res| {
            let entry = res?;
            if entry.file_type()?.is_dir() {
                get_tree_structure(entry.path(), config)
            } else {
                Ok(VCSTree::File(
                    VCSFile {
                        name: entry
                            .file_name()
                            .into_string()
                            .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid"))?,
                        sha: calculate_hash(entry.path()).map_err(|_| {
                            VCSError::IOError(
                                format!("Unable to calculate hash for {:?}", entry.path()).into(),
                            )
                        })?,
                    },
                    VCSKind::New,
                ))
            }
        })
        .collect::<VCSResult<Vec<VCSTree>>>()?;

    Ok(VCSTree::Root(VCSDirectory {
        name: root
            .file_name()
            .unwrap_or(&OsString::from("."))
            .to_str()
            .unwrap()
            .to_string(),
        children: dir_contents,
    }))
}

fn calculate_hash(path: PathBuf) -> Result<Box<[u8]>, Error> {
    let contents = fs::read_to_string(path)?;
    let mut hasher = Sha256::new();
    hasher.update(contents);
    let result = hasher.finalize();
    Ok(result[..].into())
}
