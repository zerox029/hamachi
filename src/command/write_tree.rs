use crate::command::hash_object::hash_object;
use crate::object::tree::{Entry, Mode};
use crate::object::{Hash, Object, ObjectType};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

pub(crate) fn write_tree(path_buf: Option<PathBuf>) -> std::io::Result<Hash> {
    let path = path_buf.unwrap_or(PathBuf::from("."));

    let paths_iter = fs::read_dir(&path)?;
    let paths = paths_iter
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .collect::<Vec<_>>();

    let mut entries = Vec::new();
    for path in paths {
        let metadata = fs::metadata(&path)?;

        if metadata.is_file() {
            let hash: Hash = hash_object(true, &path)?;

            let entry = Entry {
                mode: Mode::REGULAR,
                filename: path.file_name().unwrap().to_string_lossy().to_string(),
                object_type: ObjectType::BLOB,
                hash,
            };
            entries.push(entry);
        } else {
            if &path == &PathBuf::from("./.git") || &path == Path::new("./.hamachi") {
                continue;
            }

            let hash = write_tree(Some(PathBuf::from(&path)))?;

            let entry = Entry {
                mode: Mode::DIRECTORY,
                filename: path.file_name().unwrap().to_string_lossy().to_string(),
                object_type: ObjectType::TREE,
                hash,
            };
            entries.push(entry);
        }
    }

    let mut entry_byte_vectors = Vec::new();
    for mut entry in entries {
        let mut entry_bytes = format!("{} {}\0", entry.mode as u32, entry.filename)
            .as_bytes()
            .to_vec();
        entry_bytes.append(&mut entry.hash.0);

        entry_byte_vectors.push(entry_bytes);
    }

    let entries_section = entry_byte_vectors
        .into_iter()
        .flatten()
        .collect::<Vec<u8>>();
    let header = format!("tree {}\0", entries_section.len())
        .as_bytes()
        .to_vec();

    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, &header);
    Digest::update(&mut hasher, &entries_section);
    let hash = Hash(hasher.finalize().as_slice().to_vec());

    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    compressor.write_all(&header)?;
    compressor.write_all(&entries_section)?;
    let compressed_bytes = compressor.finish()?;

    Object::write_to_disk(&hash, &compressed_bytes)?;

    Ok(hash)
}
#[cfg(test)]
mod tests {
    use crate::command::write_tree::write_tree;
    use crate::object::Object;
    use crate::test_utils::*;
    use rusty_fork::rusty_fork_test;
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use std::process::Command;

    rusty_fork_test! {
        #[test]
        fn write_tree_non_recursive_test() {
            // Setup
            let repo = setup_test_environment().unwrap();

            let test_file_path = "test.txt";
            File::create(&test_file_path).unwrap();
            fs::write(&test_file_path, "this is some test content").unwrap();

            let test_file_two_path = "test2.txt";
            File::create(&test_file_two_path).unwrap();
            fs::write(&test_file_path, "this is more test content").unwrap();

            run_git_command(Command::new("git").arg("add").arg(".")).unwrap();

            // Test
            let expected_tree_hash = run_git_command(Command::new("git").arg("write-tree")).unwrap();
            let actual_tree_hash = write_tree(None).unwrap().to_string();

            let actual_tree_content = Object::decompress_object(&actual_tree_hash, false).unwrap();
            let expected_tree_content = Object::decompress_object(&expected_tree_hash, true).unwrap();

            assert_eq!(expected_tree_hash, actual_tree_hash);
            assert_eq!(expected_tree_content, actual_tree_content);

            teardown(repo).unwrap();
        }
    }

    rusty_fork_test! {
        #[test]
        fn write_tree_recursive_test() {
            // Setup
            let repo = setup_test_environment().unwrap();

            let test_file_path = "test.txt";
            let _ = File::create(&test_file_path).unwrap();
            fs::write(&test_file_path, "this is some test content").unwrap();

            let test_dir_path = "testdir";
            fs::create_dir(&test_dir_path).unwrap();

            let test_file_two_path = PathBuf::from(test_dir_path).join("test2.txt");
            File::create(test_file_two_path).unwrap();

            run_git_command(Command::new("git").arg("add").arg(".")).unwrap();

            // Test
            let expected_tree_hash = run_git_command(Command::new("git").arg("write-tree")).unwrap();
            let actual_tree_hash = write_tree(None).unwrap().to_string();

            let actual_tree_content = Object::decompress_object(&actual_tree_hash, false).unwrap();
            let expected_tree_content = Object::decompress_object(&expected_tree_hash, true).unwrap();

            assert_eq!(expected_tree_hash, actual_tree_hash);
            assert_eq!(expected_tree_content, actual_tree_content);

            teardown(repo).unwrap();
        }
    }
}
