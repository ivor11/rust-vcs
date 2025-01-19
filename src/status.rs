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
            let commit_id = matched_commit.file_name();
            let mut path = PathBuf::from("./.rust-vcs/commits");
            path.push(commit_id);

            let mut old_tree_path = path.clone();
            old_tree_path.push("meta");
            old_tree_path.push("tree.json");

            let old_tree: VCSTree = serde_json::from_str(&fs::read_to_string(old_tree_path)?)?;

            let diff = tree.diff_tree(old_tree);

            match diff {
                None => println!("No changes to commit"),
                Some(t) => {
                    println!("Changes:");
                    print!("{}", t.to_string().yellow());
                }
            }
        }
    };
    // .unwrap_or(Err(Error::new(ErrorKind::Other, "unable to find commit")))?;

    Ok(())
}

pub fn get_tree_structure(root: PathBuf) -> Result<VCSTree, Error> {
    let ignore: [&str; 4] = [".rust-vcs", "target", "Cargo.lock", ".git"];
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
    Root(VCSDirectory),
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
            Self::Directory(d) | Self::Root(d) => d
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
        VCSTree::Root(VCSDirectory {
            name: path.into_os_string().to_str().unwrap().to_string(),
            children: contents,
        })
    }

    fn copy_contents(&self, mut from: PathBuf, mut to: PathBuf) -> Result<(), Error> {
        match self {
            Self::File(f) => {
                fs::copy(
                    format!(
                        "{}/{}",
                        from.into_os_string()
                            .into_string()
                            .expect("Unable to unpack from"),
                        f.name
                    ),
                    format!(
                        "{}/{}",
                        to.into_os_string()
                            .into_string()
                            .expect("Unable to unpack to"),
                        f.name
                    ),
                )?;
                Ok(())
            }
            Self::Directory(d) | Self::Root(d) => {
                from.push(d.name.clone());
                to.push(d.name.clone());
                fs::create_dir_all(to.clone())?;
                println!(
                    "{}",
                    to.clone()
                        .into_os_string()
                        .into_string()
                        .expect("Unable to unpack to")
                );
                for child in d.children.clone() {
                    child.copy_contents(from.clone(), to.clone())?;
                }
                Ok(())
            }
        }
    }

    pub fn copy_to(&self, new_path: PathBuf) -> Result<Self, Error> {
        let root_directory: VCSDirectory = self.get_root_dir();

        self.copy_contents(PathBuf::from(&root_directory.name), new_path.clone())?;

        let new_tree = self.form_tree(new_path, root_directory.children);

        Ok(new_tree)
    }

    // fn get_to_contents(&self, path: PathBuf) -> Result<Vec<Self>, Error> {

    //     let root_directory: VCSDirectory = match self {
    //         Self::File(f) => {
    //             panic!("Unable to parse root directory")
    //         }
    //         Self::Directory(d) => d.clone(),
    //     };

    //     let mut tree = self.clone();

    //     for p in path.iter() {
    //         match tree {
    //             Self::File(f) => {
    //                 panic!("Unable to parse directory")
    //             }
    //             Self::Directory(d) => {
    //                 tree = d.children.into_iter().filter(|child| match child.name == p).last().expect("Unable to parse directory")
    //             },
    //         }
    //     }

    //     let root_directory: VCSDirectory = match self {
    //         Self::File(f) => {
    //             panic!("Unable to parse directory")
    //         }
    //         Self::Directory(d) => d.clone(),
    //     };

    // }

    fn get_root_dir(&self) -> VCSDirectory {
        match self {
            Self::File(_) | Self::Directory(_) => {
                panic!("Unable to parse root directory")
            }
            Self::Root(d) => d.clone(),
        }
    }

    fn diff_tree(&self, old_tree: Self) -> Option<Self> {
        match (self, old_tree) {
            (Self::File(a), Self::File(b)) => {
                if a.sha == b.sha {
                    None
                } else {
                    Some(self.clone())
                }
            }
            (Self::Directory(a), Self::Directory(b)) | (Self::Root(a), Self::Root(b)) => {
                let vec = a
                    .children
                    .iter()
                    .zip(b.children.iter())
                    .map(|(c, o)| c.clone().diff_tree(o.clone()))
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap())
                    .collect::<Vec<VCSTree>>();

                if vec.len() > 0 {
                    let diff_dir = VCSDirectory {
                        children: vec,
                        ..a.clone()
                    };
                    match self {
                        Self::Directory(_) => Some(Self::Directory(diff_dir)),
                        Self::Root(_) => Some(Self::Root(diff_dir)),
                        _ => None,
                    }
                } else {
                    None
                }
            }
            (_, _) => panic!("Unmatched diff terms"),
        }
    }
}
