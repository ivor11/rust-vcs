use super::error::{VCSError, VCSResult};
use super::status;
use super::util;
use std::fmt;
use std::fs::{self, File};
use std::hash::{DefaultHasher, Hash, Hasher};
use std::io::Write;
use std::path::PathBuf;
use std::time::SystemTime;

fn calculate_hash<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    format!("{:x}", s.finish())
}

#[derive(Hash)]
struct Commit {
    id: String,
    message: String,
    time: SystemTime,
}

impl Commit {
    fn new(message: String) -> Self {
        let commit_time = SystemTime::now();
        Self {
            id: calculate_hash(&commit_time),
            message,
            time: commit_time,
        }
    }
}

impl fmt::Display for Commit {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}\t{}\t{}",
            self.id,
            util::systemtime_strftime(self.time, "%Y/%m/%d %T"),
            self.message
        )
    }
}

pub fn commit(message: String) -> VCSResult<()> {
    fs::exists(".rust-vcs/index").map(|x| {
        if !x {
            Err(VCSError::Uninitialized)
        } else {
            Ok(())
        }
    })??;

    status::get_current_diff_tree()?.ok_or(VCSError::Other("No changes to commit".into()))?;

    // println!("{:?}", x);
    let tree = status::get_tree_structure(".".into())?;

    let commit = Commit::new(message);

    let commit_root = format!(".rust-vcs/commits/{}", commit.id);
    fs::create_dir_all(format!("{}/meta", commit_root))
        .and_then(|_| fs::create_dir_all(format!("{}/data", commit_root)))?;

    let json_data =
        serde_json::to_string(&tree.copy_to(PathBuf::from(format!("{}/data", commit_root)))?)
            .unwrap();
    fs::write(format!("{}/meta/tree.json", commit_root), json_data)?;

    fs::write(".rust-vcs/current", commit.id.to_string())?;

    let mut logfile = File::options().append(true).open(".rust-vcs/index")?;
    writeln!(&mut logfile, "{}", commit)?;

    Ok(())
}
