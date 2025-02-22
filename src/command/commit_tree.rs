use crate::object::commit::Commit;
use crate::object::{Hash, Object};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::io::Write;
use std::str::FromStr;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn commit_tree(hash: &str, message: &Option<String>) -> std::io::Result<Hash> {
    let default_author_name = String::from("Osamu Dazai");
    let default_author_email = String::from("osamu.dazai@gmail.com");
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();
    let default_timezone = String::from("-0500");

    let commit = Commit {
        tree_hash: Hash::from_str(&hash).expect("Invalid hash"),
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

    let commit_file_content = commit.to_object_file_representation();

    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, &commit_file_content);
    let hash = Hash(hasher.finalize().as_slice().to_vec());

    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    compressor.write_all(&commit_file_content)?;
    let compressed_bytes = compressor.finish()?;

    Object::write_to_disk(&hash, &compressed_bytes)?;

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use crate::command::commit_tree::commit_tree;
    use crate::object::Object;
    use crate::test_utils::{run_git_command, setup_test_environment, teardown};
    use std::fs::File;
    use std::process::Command;

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
        let expected_hash = run_git_command(
            Command::new("git")
                .arg("commit-tree")
                .arg(&tree_hash)
                .arg("-m")
                .arg(commit_message),
        )
        .unwrap();
        let actual_hash = commit_tree(&tree_hash, &Some(String::from(commit_message)))
            .unwrap()
            .to_string();

        let expected_content = Object::decompress_object(&expected_hash, true).unwrap();
        let actual_content = Object::decompress_object(&actual_hash, false).unwrap();

        assert_eq!(actual_hash, expected_hash);
        assert_eq!(actual_content, expected_content);

        teardown(repo).unwrap();
    }
}
