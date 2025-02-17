use std::fmt::Display;
use crate::object::{Hash};

pub(crate) struct Commit {
    pub(crate) tree_hash: Hash,
    pub(crate) parents: Vec<Parent>,
    pub(crate) author_name: String,
    pub(crate) author_email: String,
    pub(crate) author_date: u64,
    pub(crate) author_date_timezone: String,
    pub(crate) committer_name: String,
    pub(crate) committer_email: String,
    pub(crate) committer_date: u64,
    pub(crate) committer_date_timezone: String,
    pub(crate) commit_message: String,
}

pub(crate) struct Parent {
    pub(crate) parent_hash: Hash,
}
impl Display for Parent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("parent {}", self.parent_hash.to_string()))
    }
}