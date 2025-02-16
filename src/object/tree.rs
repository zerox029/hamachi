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
use crate::object::{Object, ObjectType};
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
    let entry = String::new();

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
        hash: hex::encode(entry_buffer),
    };

    println!("{}", entry);

    Ok((entry, read_bytes))
}

pub(crate) fn write_tree(path_buf: Option<PathBuf>) -> std::io::Result<String> {
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
            let hash = hash_object(true, &path)?;

            let entry = Entry{
                mode: Mode::REGULAR,
                filename: path.file_name().unwrap().to_string_lossy().to_string(),
                object_type: ObjectType::BLOB,
                hash,
            };
            entries.push(entry);
        }
        else {
            if &path == &PathBuf::from("./.git") || &path == Path::new("./target") || &path == Path::new("./hamachi") {
                continue;
            }

            let hash = write_tree(Some(PathBuf::from(&path)))?;

            let entry = Entry{
                mode: Mode::REGULAR,
                filename: path.file_name().unwrap().to_string_lossy().to_string(),
                object_type: ObjectType::BLOB,
                hash,
            };
            entries.push(entry);
        }
    }

    let mut entry_strings = Vec::new();
    for entry in entries {
        let entry_string = format!("{} {}\0{}", entry.mode as u32, entry.filename, entry.hash);
        entry_strings.push(entry_string);
    }
    let entries_section = entry_strings.join("");
    let header = format!("tree {}\0{}", entry_strings.len(), entries_section);

    let tree_content = format!("{}\0{}", header, entries_section);

    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, &header);
    let hash = hex::encode(hasher.finalize());
    
    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    compressor.write_all(&tree_content.into_bytes())?;
    let compressed_bytes = compressor.finish()?;
    
    Object::write_to_disk(&hash, &compressed_bytes)?;
    
    Ok(hash)
}

#[derive(Debug)]
struct Entry {
    mode: Mode,
    filename: String,
    object_type: ObjectType,
    hash: String,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:0>6} {} {}\t{}", self.mode as u32, self.object_type, self.hash, self.filename)
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
    use std::io::Write;
    use std::process::{Command, Stdio};
    use rusty_fork::rusty_fork_test;
    use crate::object::tree::ls_tree;
    use crate::test_utils::{copy_git_object_file, run_git_command, run_git_command_piped_input, setup_test_environment};

        #[test]
        fn ls_tree_blob_only() {
            // Setup
            setup_test_environment().unwrap();

            let test_file_path = "test.txt";
            let test_file = File::create(&test_file_path).unwrap();
            fs::write(&test_file_path, "this is some test content").unwrap();

            let file_hash = run_git_command(Command::new("git").arg("hash-object").arg("-w").arg(&test_file_path))
                .expect("Failed to hash object");

            let entry = format!("100644 blob {file_hash}\t{test_file_path}");
            let mktree_command = Command::new("git").arg("mktree").stdin(Stdio::piped()).stdout(Stdio::piped()).spawn().unwrap();
            let tree_hash = run_git_command_piped_input(mktree_command, entry).unwrap();
            
            copy_git_object_file(&file_hash).unwrap();
            copy_git_object_file(&tree_hash).unwrap();

            // Test
            let expected = run_git_command(Command::new("git").arg("ls-tree").arg(&tree_hash)).unwrap();
            let actual = ls_tree(false, &tree_hash).unwrap();
            
            assert_eq!(expected, actual);
    }
}