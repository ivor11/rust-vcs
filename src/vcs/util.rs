use chrono::offset::Utc;
use chrono::DateTime;
use std::fs;
use super::error::{VCSError, VCSResult};
use std::hash::{DefaultHasher, Hash, Hasher};

pub fn systemtime_strftime<T>(dt: T, format: &str) -> String
where
    T: Into<DateTime<Utc>>,
{
    dt.into().format(format).to_string()
}

pub fn calculate_hash<T: Hash>(t: &T) -> String {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    format!("{:x}", s.finish())
}

pub fn check_vcs_initialized() -> VCSResult<()> {
    fs::exists(".rust-vcs/index").map(|x| {
        if !x {
            Err(VCSError::Uninitialized)
        } else {
            Ok(())
        }
    })?
}