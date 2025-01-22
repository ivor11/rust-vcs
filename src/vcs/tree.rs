use super::error::VCSResult;
use colored::{ColoredString, Colorize};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Clone)]
pub struct VCSFile {
    pub name: String,
    pub sha: Box<[u8]>,
}

impl Eq for VCSFile {}

impl PartialEq for VCSFile {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for VCSFile {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, Hash)]
pub enum VCSKind {
    New,
    Deleted,
    Modified,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct VCSDirectory {
    pub name: String,
    pub children: Vec<VCSTree>,
}

impl Eq for VCSDirectory {}

impl PartialEq for VCSDirectory {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Hash for VCSDirectory {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state);
    }
}

#[derive(Serialize, Deserialize, Clone, Hash, PartialEq, Eq)]
pub enum VCSTree {
    Root(VCSDirectory),
    File(VCSFile, VCSKind),
    Directory(VCSDirectory),
}

impl VCSTree {
    pub fn to_string(&self) -> ColoredString {
        self.paths_with_prefix(None)
            .into_iter()
            .reduce(|acc, f| format!("{}\n{}", acc, f).into())
            .unwrap_or("None".to_string().into())
    }

    fn paths_with_prefix(&self, path: Option<String>) -> Vec<ColoredString> {
        let root = path.unwrap_or("".to_string());
        match self {
            Self::File(f, k) => {
                let content = format!("{}{}", root, f.name);
                let content_colored = match k {
                    VCSKind::New => content.green(),
                    VCSKind::Modified => content.yellow(),
                    VCSKind::Deleted => content.red(),
                };
                Vec::from([content_colored.into()])
            }
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

    fn form_tree(&self, path: PathBuf, contents: Vec<VCSTree>) -> Self {
        VCSTree::Root(VCSDirectory {
            name: path.into_os_string().to_str().unwrap().to_string(),
            children: contents,
        })
    }

    fn copy_contents(&self, mut from: PathBuf, mut to: PathBuf) -> VCSResult<()> {
        match self {
            Self::File(f, _) => {
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

    pub fn copy_to(&self, new_path: PathBuf) -> VCSResult<Self> {
        let root_directory: VCSDirectory = self.get_root_dir();

        self.copy_contents(PathBuf::from(&root_directory.name), new_path.clone())?;

        let new_tree = self.form_tree(new_path, root_directory.children);

        Ok(new_tree)
    }

    fn get_root_dir(&self) -> VCSDirectory {
        match self {
            Self::File(_, _) | Self::Directory(_) => {
                panic!("Unable to parse root directory")
            }
            Self::Root(d) => d.clone(),
        }
    }

    fn set_file_kind(&self, kind: VCSKind) -> Self {
        match self {
            Self::File(x, _) => Self::File(x.clone(), kind),
            Self::Directory(x) => Self::Directory(VCSDirectory {
                children: x
                    .children
                    .clone()
                    .into_iter()
                    .map(|x| x.set_file_kind(kind.clone()))
                    .collect(),
                ..x.clone()
            }),
            Self::Root(x) => Self::Root(VCSDirectory {
                children: x
                    .children
                    .clone()
                    .into_iter()
                    .map(|x| x.set_file_kind(kind.clone()))
                    .collect(),
                ..x.clone()
            }),
        }
    }

    pub fn diff_tree(&self, old_tree: Self) -> Option<Self> {
        match (self, old_tree) {
            (Self::File(a, _), Self::File(b, _)) => {
                if a.sha == b.sha {
                    None
                } else {
                    Some(Self::File(a.clone(), VCSKind::Modified))
                }
            }
            (Self::Directory(a), Self::Directory(b)) | (Self::Root(a), Self::Root(b)) => {
                let aset: HashSet<VCSTree> = a.children.iter().cloned().collect();
                let bset: HashSet<VCSTree> = b.children.iter().cloned().collect();

                let intersection_elements: Vec<VCSTree> =
                    aset.intersection(&bset).cloned().collect();
                let new_elements: Vec<VCSTree> = aset
                    .difference(&intersection_elements.iter().cloned().collect())
                    .cloned()
                    .collect();
                let deleted_elements: Vec<VCSTree> = bset
                    .difference(&intersection_elements.iter().cloned().collect())
                    .cloned()
                    .collect();

                let mut modified_vec = a
                    .children
                    .clone()
                    .iter()
                    .filter(|x| intersection_elements.contains(x))
                    .zip(
                        b.children
                            .iter()
                            .filter(|x| intersection_elements.contains(x)),
                    )
                    .map(|(c, o)| c.clone().diff_tree(o.clone()))
                    .filter(|x| x.is_some())
                    .map(|x| x.unwrap())
                    .collect::<Vec<VCSTree>>();

                let new_vec: Vec<VCSTree> = a
                    .children
                    .iter()
                    .filter(|x| new_elements.contains(x))
                    .map(|x| x.set_file_kind(VCSKind::New))
                    .collect();

                let deleted_vec: Vec<VCSTree> = b
                    .children
                    .iter()
                    .filter(|x| deleted_elements.contains(x))
                    .map(|x| x.set_file_kind(VCSKind::Deleted))
                    .collect();

                modified_vec.extend(new_vec);
                modified_vec.extend(deleted_vec);

                if modified_vec.len() > 0 {
                    let diff_dir = VCSDirectory {
                        children: modified_vec,
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
