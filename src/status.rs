use clap::error::Result;
use colored::Colorize;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::ffi::OsString;
use std::fs::{self, File};
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

pub fn status() -> Result<(), Error> {
    let current_commit = fs::read_to_string(".rust-vcs/current")?;

    let matched_commit = fs::read_dir(".rust-vcs/commits")?
        .filter(|dir| {
            dir.as_ref().expect("match fail").file_name().into_string()
                == Ok(current_commit.to_owned())
        })
        .last();

    let tree = get_tree_structure(".".into())?;
    match matched_commit {
        None => {
            //new instance
            println!("New VCS Repository: Untracked files");
            print!("{}", tree.to_string().green());
        }
        Some(matched_value) => {
            //old instance
            let matched_commit = matched_value?;
        }
    };
    // .unwrap_or(Err(Error::new(ErrorKind::Other, "unable to find commit")))?;

    Ok(())
}

pub fn get_tree_structure(root: PathBuf) -> Result<VCSTree, Error> {
    let ignore = [".rust-vcs", "target", "Cargo.lock", ".git"];
    let dir_contents = fs::read_dir(&root)?
        .filter(|res| {
            res.as_ref()
                .map(|entry| !ignore.contains(&entry.file_name().to_str().unwrap_or("")))
                .expect("FAILED TO FILTER")
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

#[derive(Serialize, Deserialize, Clone)]
pub struct VCSFile {
    name: String,
    sha: Box<[u8]>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VCSDirectory {
    name: String,
    children: Vec<VCSTree>,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum VCSTree {
    File(VCSFile),
    Directory(VCSDirectory),
}

impl VCSTree {
    fn to_string(&self) -> String {
        self.paths_with_prefix(None)
            .into_iter()
            .reduce(|acc, f| format!("{}\n{}", acc, f))
            .unwrap_or("None".to_string())
    }

    fn paths_with_prefix(&self, path: Option<String>) -> Vec<String> {
        let root = path.unwrap_or("".to_string());
        match self {
            Self::File(f) => Vec::from([format!("{}{}", root, f.name)]),
            Self::Directory(d) => d
                .children
                .iter()
                .map(|f| f.paths_with_prefix(Some(format!("{root}{}/", d.name))))
                .reduce(|mut acc, f| {
                    acc.extend(f);
                    acc
                })
                .unwrap_or(Vec::new()),
        }
    }

    fn form_tree(&self, mut path: PathBuf, contents: Vec<VCSTree>) -> Self {
        let new_tree = VCSTree::Directory(VCSDirectory {
            name: path
                .file_name()
                .unwrap_or(&OsString::from("None"))
                .to_str()
                .unwrap()
                .to_string(),
            children: contents,
        });

        if path.pop() && path.file_name().is_some() {
            self.form_tree(path, Vec::from([new_tree]))
        } else {
            new_tree
        }
    }

    fn copy_contents(&self, mut from: PathBuf, mut to: PathBuf) -> Result<(), Error>{
        match self {
            Self::File(f) => {
                fs::copy(format!("{}/{}", from.into_os_string().into_string().expect("Unable to unpack from"), f.name), format!("{}/{}",to.into_os_string().into_string().expect("Unable to unpack to"), f.name))?;
                Ok(())
            }
            Self::Directory(d) => {
                from.push(d.name.clone());
                to.push(d.name.clone());
                fs::create_dir_all(to.clone())?;
                println!("{}", to.clone().into_os_string().into_string().expect("Unable to unpack to"));
                for child in d.children.clone() {
                    child.copy_contents(from.clone(), to.clone())?;
                }
                Ok(())
            }
        }
    }

    pub fn copy_to(&self, new_root: String) -> Self {
        let root_directory: VCSDirectory = match self {
            Self::File(f) => {
                panic!("Unable to parse root directory")
            }
            Self::Directory(d) => d.clone(),
        };

        let new_path: PathBuf = PathBuf::from(new_root);

        for content in &root_directory.children {
            content.copy_contents(PathBuf::from(&root_directory.name), new_path.clone());
        }

        let new_tree = self.form_tree(new_path, root_directory.children);

        new_tree
    }
}
