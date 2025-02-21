use std::str::FromStr;
use std::fmt::Display;
use std::io::Read;
use flate2::read::ZlibDecoder;
use crate::object::{Hash};

#[derive(Debug)]
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

impl Commit {
    pub(crate) fn from_compressed_data(data: &[u8]) -> (Self, usize) {
        let mut decompressor = ZlibDecoder::new(data);
        let mut decompressed_data = String::new();
        decompressor.read_to_string(&mut decompressed_data).unwrap();

        let mut lines = decompressed_data.lines();

        // Tree hash
        let (line_type, tree_hash) = lines.next().unwrap().split_once(' ').unwrap();
        let tree_hash = Hash::from_str(tree_hash).unwrap();
        assert_eq!(line_type, "tree");

        // Parent hash
        // TODO: Support multiple parents
        let parent_line = lines.next().unwrap();
        let parent_hash = if parent_line.starts_with("parent") {
            let parent_hash = parent_line.split_once(' ').unwrap().1;

            Some(Hash::from_str(parent_hash).unwrap())
        } else {
            None
        };
        let parents = if parent_hash.is_some() { vec![Parent::new(parent_hash.unwrap())] } else { Vec::new() };

        // Author
        let mut author_line = if parents.len() > 0 { lines.next().unwrap() } else { parent_line };
        assert!(author_line.starts_with("author"));
        let email_start = author_line.find("<").unwrap();
        let email_end = author_line.find(">").unwrap();
        let author_name = author_line["author ".len()..email_start].trim().to_string();
        let author_email = author_line[email_start + 1..email_end].trim().to_string();

        let mut remaining = author_line[email_end + 1..].trim().split_whitespace();
        let author_date = remaining.next().unwrap();
        let author_date = author_date.parse::<u64>().unwrap();
        let author_date_timezone = remaining.next().unwrap().to_string();

        // Committer
        let mut committer_line = lines.next().unwrap();
        assert!(committer_line.starts_with("committer"));
        let email_start = committer_line.find("<").unwrap();
        let email_end = committer_line.find(">").unwrap();
        let committer_name = committer_line["committer ".len()..email_start].trim().to_string();
        let committer_email = committer_line[email_start + 1..email_end].trim().to_string();

        let mut remaining = committer_line[email_end + 1..].trim().split_whitespace();
        let committer_date = remaining.next().unwrap();
        let committer_date = committer_date.parse::<u64>().unwrap();
        let committer_date_timezone = remaining.next().unwrap().to_string();

        let commit_message = lines.skip(1).next().unwrap().to_string();

        (Self {
            tree_hash,
            parents,
            author_name,
            author_email,
            author_date,
            author_date_timezone,
            committer_name,
            committer_email,
            committer_date,
            committer_date_timezone,
            commit_message,
        }, decompressor.total_in() as usize)
    }

    pub(crate) fn to_object_file_representation(&self) -> Vec<u8> {
        let content = format!("tree {}{}\nauthor {} <{}> {} {}\ncommitter {} <{}> {} {}\n\n{}\n",
                              self.tree_hash.to_string(),
                              self.parents.iter().map(|p| format!("\nparent {}", p.parent_hash.to_string())).collect::<Vec<String>>().join(""),
                              self.author_name,
                              self.author_email,
                              self.author_date,
                              self.author_date_timezone,
                              self.committer_name,
                              self.committer_email,
                              self.committer_date,
                              self.committer_date_timezone,
                              self.commit_message);
        let header = format!("commit {}", content.len());
        let commit_string = format!("{}\0{}", header, content);
        
        commit_string.as_bytes().to_vec()
    }
}

#[derive(Debug)]
pub(crate) struct Parent {
    pub(crate) parent_hash: Hash,
}

impl Parent {
    pub(crate) fn new(hash: Hash) -> Self {
        Self {
            parent_hash: hash
        }
    }
}

impl Display for Parent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", format!("parent {}", self.parent_hash.to_string()))
    }
}