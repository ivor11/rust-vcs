use clap::error::Result;
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fmt::format;
use std::fs::{self, DirEntry};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

pub fn status() -> Result<(), Error> {
    let current_commit = fs::read_to_string(".rust-vcs/current")?;

    let matched_commit = fs::read_dir(".rust-vcs/commits")?
        .filter(|dir| {
            dir.as_ref().expect("match fail").file_name().into_string() == Ok(current_commit.to_owned())
        })
        .last();

    let tree = get_tree_structure(".".into())?;
    match matched_commit {
        None => {
            println!("");
            print!("{}", tree.to_string());
            //new instance
        }
        Some(matched_value) => {
            //old instance
            let matched_commit = matched_value?;
        }
    };
    // .unwrap_or(Err(Error::new(ErrorKind::Other, "unable to find commit")))?;

    Ok(())
}

fn get_tree_structure(root: PathBuf) -> Result<VCSTree, Error> {
    let ignore = [".rust-vcs", "target", "Cargo.lock", ".git"];
    let dir_contents = fs::read_dir(&root)?
        .filter(|res| {
            res.as_ref().map(|entry| !ignore.contains(&entry
                .file_name().to_str().unwrap_or(""))).expect("FAILED TO FILTER")
        })
        .map(|res| {
            let entry = res?;
            if entry.file_type()?.is_dir() {
                get_tree_structure(entry.path())
            } else {
                Ok(VCSTree::File(VCSFile {
                    name: entry
                        .file_name()
                        .into_string()
                        .map_err(|_| Error::new(ErrorKind::InvalidData, "invalid"))?,
                    sha: calculate_hash(entry.path())?,
                }))
            }
        })
        .collect::<Result<Vec<VCSTree>, Error>>()?;

    Ok(VCSTree::Directory(VCSDirectory {
        name: root.file_name().unwrap_or(&OsString::from("-")).to_str().unwrap().to_string(),
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

struct VCSFile {
    name: String,
    sha: Box<[u8]>,
}

struct VCSDirectory {
    name: String,
    children: Vec<VCSTree>,
}

enum VCSTree {
    File(VCSFile),
    Directory(VCSDirectory),
}

impl VCSTree {
    fn to_string(&self) -> String {
        match self {
            Self::File(f) => {
                format!("{}", f.name)
            }
            Self::Directory(d) => {
                format!(
                    "{}\n{}",
                    d.name,
                    d.children
                        .iter()
                        .map(|f| format!("\t{}", f.to_string()))
                        .reduce(|acc, f| format!("{}\n{}", acc, f)).unwrap_or("None".to_string())

                )
            }
        }
    }
}
