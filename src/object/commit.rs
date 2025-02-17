use std::io::Write;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use crate::object::{Hash, Object};

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

pub fn commit_tree(hash: &str, message: &Option<String>) -> std::io::Result<Hash> {
    let default_author_name = String::from("Osamu Dazai");
    let default_author_email = String::from("osamu.dazai@gmail.com");
    let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("Time went backwards").as_secs();
    let default_timezone = String::from("-0500");

    let commit = Commit {
        tree_hash:  Hash::from_str(&hash).expect("Invalid hash"),
        parents: Vec::new(),
        author_name: default_author_name.clone(),
        author_email: default_author_email.clone(),
        author_date: now,
        author_date_timezone: default_timezone.clone(),
        committer_name: default_author_name.clone(),
        committer_email: default_author_email.clone(),
        committer_date: now,
        committer_date_timezone: default_timezone.clone(),
        commit_message: message.clone().unwrap_or_default().to_string(),
    };
    
    let content = format!("tree {}{}\nauthor {} <{}> {} {}\ncommitter {} <{}> {} {}\n\n{}\n",
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

    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, commit_string.as_bytes());
    let hash = Hash(hasher.finalize().as_slice().to_vec());

    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    compressor.write_all(&commit_string.as_bytes())?;
    let compressed_bytes = compressor.finish()?;

    Object::write_to_disk(&hash, &compressed_bytes)?;
    
    Ok(hash)
}

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::process::Command;
    use crate::object::commit::commit_tree;
    use crate::object::{Object};
    use crate::test_utils::{run_git_command, setup_test_environment, teardown};

    #[test]
    fn commit_tree_test() {
        // Setup
        let repo = setup_test_environment().unwrap();

        let test_file_path = "test.txt";
        File::create(test_file_path).unwrap();

        run_git_command(Command::new("git").arg("add").arg(".")).unwrap();
        let tree_hash = run_git_command(Command::new("git").arg("write-tree")).unwrap();

        // Test
        let commit_message = "this is a commit message";
        let expected_hash = run_git_command(Command::new("git").arg("commit-tree").arg(&tree_hash).arg("-m").arg(commit_message)).unwrap();
        let actual_hash = commit_tree(&tree_hash, &Some(String::from(commit_message))).unwrap().to_string();

        let expected_content = Object::decompress_object(&expected_hash, true).unwrap();
        let actual_content = Object::decompress_object(&actual_hash, false).unwrap();

        assert_eq!(actual_hash, expected_hash);
        assert_eq!(actual_content, expected_content);
        
        teardown(repo).unwrap();
    }
}