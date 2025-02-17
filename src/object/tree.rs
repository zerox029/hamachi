use std::cmp::PartialEq;
use std::ffi::{CStr};
use std::fmt::{Display};
use std::fs;
use std::io::{BufRead, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use crate::object::{Hash, Object, ObjectType};
use crate::object::blob::hash_object;

/// List the contents of a tree object with the specified hash
/// https://git-scm.com/docs/git-ls-tree
pub(crate) fn ls_tree(_name_only: bool, hash: &str) -> std::io::Result<String> {
    let mut tree = Object::from_hash(hash).expect("error here lol");
    assert_eq!(tree.header.object_type, ObjectType::TREE, "Object was not a tree");

    // Read the rest of the file
    let mut read_bytes = 0;
    let mut result = String::new();
    while read_bytes < tree.header.size {
        let (entry, size) = get_current_tree_entry(&mut tree).expect("error reading entry");
        read_bytes += size;
        
        result.push_str(entry.to_string().as_str());
    }

    Ok(result)
}

/// Returns the tree entry at the current position in the tree buffer reader and its size in bytes
fn get_current_tree_entry(tree: &mut Object) -> Result<(Entry, usize), &'static str> {
    let mut read_bytes = 0;

    let mut entry_buffer = Vec::new();
    tree.content_buffer_reader.read_until(b'\0', &mut entry_buffer).expect("error reading header");
    read_bytes += entry_buffer.len();

    let header_string = CStr::from_bytes_with_nul(&entry_buffer).expect("File header missing null byte");
    let header_string = header_string.to_str().expect("File header contains invalid UTF-8");

    let Some((mode, file_name)) = header_string.split_once(' ') else { panic!{"Entry missing space delimiter"} };
    let mode = mode.to_string();
    let file_name = file_name.to_string();

    entry_buffer.clear();
    entry_buffer.resize(20, 0);

    tree.content_buffer_reader.read_exact(&mut entry_buffer).expect("error reading header");
    read_bytes += entry_buffer.len();

    let mode = Mode::from_str(&mode).expect("Invalid mode");
    let entry = Entry{
        mode,
        filename: file_name.to_string(),
        object_type: if mode == Mode::DIRECTORY { ObjectType::TREE } else { ObjectType::BLOB },
        hash: Hash(entry_buffer),
    };

    Ok((entry, read_bytes))
}

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

            let entry = Entry{
                mode: Mode::REGULAR,
                filename: path.file_name().unwrap().to_string_lossy().to_string(),
                object_type: ObjectType::BLOB,
                hash,
            };
            entries.push(entry);
        }
        else {
            if &path == &PathBuf::from("./.git") || &path == Path::new("./.hamachi") {
                continue;
            }

            let hash = write_tree(Some(PathBuf::from(&path)))?;

            let entry = Entry{
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
        println!("{}", entry);
        let mut entry_bytes = format!("{} {}\0", entry.mode as u32, entry.filename).as_bytes().to_vec();
        entry_bytes.append(&mut entry.hash.0);

        entry_byte_vectors.push(entry_bytes);
    }

    let entries_section = entry_byte_vectors.into_iter().flatten().collect::<Vec<u8>>();
    let header = format!("tree {}\0", entries_section.len()).as_bytes().to_vec();

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

#[derive(Debug)]
struct Entry {
    mode: Mode,
    filename: String,
    object_type: ObjectType,
    hash: Hash,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:0>6} {} {}\t{}", self.mode as u32, self.object_type, &self.hash.to_string(), self.filename)
    }
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Debug)]
enum Mode {
    REGULAR = 100644,
    EXECUTABLE = 100755,
    SYMBOLIC = 120000,
    DIRECTORY = 40000,
}

impl FromStr for Mode {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "100644" => Ok(Mode::REGULAR),
            "100755" => Ok(Mode::EXECUTABLE),
            "120000" => Ok(Mode::SYMBOLIC),
            "40000" => Ok(Mode::DIRECTORY),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::fs::File;
    use std::path::PathBuf;
    use std::process::{Command};
    use rusty_fork::rusty_fork_test;
    use crate::object::Object;
    use crate::object::tree::{ls_tree, write_tree};
    use crate::test_utils::*;

    rusty_fork_test! {
        #[test]
        fn ls_tree_test() {
            // Setup
            let repo = setup_test_environment().unwrap();

            let test_file_path = "test.txt";
            let _ = File::create(&test_file_path).unwrap();
            fs::write(&test_file_path, "this is some test content").unwrap();

            let test_dir_path = "testdir";
            fs::create_dir(&test_dir_path).unwrap();

            run_git_command(Command::new("git").arg("add").arg(".")).unwrap();
            let tree_hash = run_git_command(Command::new("git").arg("write-tree")).unwrap();

            copy_git_object_file(&tree_hash).unwrap();

            // Test
            let expected = run_git_command(Command::new("git").arg("ls-tree").arg(&tree_hash)).unwrap();
            let actual = ls_tree(false, &tree_hash).unwrap();

            assert_eq!(expected, actual);

            teardown(repo).unwrap();
        }
    }

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

            println!("{}", repo.to_str().unwrap());
            println!("{}", expected_tree_hash);

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

            println!("{}", repo.to_str().unwrap());
            println!("{}", expected_tree_hash);

            assert_eq!(expected_tree_hash, actual_tree_hash);
            assert_eq!(expected_tree_content, actual_tree_content);
            
            teardown(repo).unwrap();
        }
    }
}