use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use crate::object::Hash;

struct Commit {
    tree_hash: Hash,
    parents: Vec<Parent>,
    author_name: String,
    author_email: String,
    author_date: u64,
    author_date_timezone: String,
    committer_name: String,
    committer_email: String,
    committer_date: u64,
    committer_date_timezone: String,
    commit_message: String,
}

struct Parent {
    parent_hash: Hash,
}
impl ToString for Parent {
    fn to_string(&self) -> String {
        format!("parent {}", self.parent_hash.to_string())
    }
}

pub fn commit_tree(hash: String, message: Option<String>) -> std::io::Result<Hash> {
    let default_author_name = String::from("Osamu Dazai");
    let default_author_email = String::from("osamu.dazai@gmail.com");
    let default_committer_name = String::from("Kurisu Makise");
    let default_committer_email = String::from("kurisu.makise@gadgetlab.jp");
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
    let default_timezone = String::from("+0000");

    let commit = Commit {
        tree_hash:  Hash::from_str(&hash).expect("Invalid hash"),
        parents: Vec::new(),
        author_name: default_author_name.clone(),
        author_email: default_author_email.clone(),
        author_date: now,
        author_date_timezone: default_timezone.clone(),
        committer_name: default_committer_name.clone(),
        committer_email: default_committer_email.clone(),
        committer_date: now,
        committer_date_timezone: default_timezone.clone(),
        commit_message: message.unwrap_or("".to_owned()),
    };
    
    let content = format!("tree {}{}author {} <{}> {} {}comitter {} <{}> {} {} {}", 
                          commit.tree_hash.to_string(), 
                          commit.parents.into_iter().map(|p| p.parent_hash.to_string()).collect::<Vec<String>>().join(""), 
                          commit.author_name, 
                          commit.author_email,
                          commit.author_date,
                          commit.author_date_timezone, 
                          commit.committer_name,
                          commit.committer_email,
                          commit.committer_date,
                          commit.committer_date_timezone, 
                          commit.commit_message);
    let header = format!("commit {}", content.len());
    let commit_string = format!("{}\0{}", header, content);

    println!("{}", commit_string);

    Ok(Hash::from_str(&hash).expect("Invalid hash"))
}