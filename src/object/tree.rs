use std::cmp::PartialEq;
use std::ffi::{CStr, OsStr};
use std::fmt::{format, Display};
use std::fs;
use std::io::{BufRead, Read};
use std::path::{Path, PathBuf};
use std::str::FromStr;
use crate::object::{Object, ObjectType};
use crate::object::blob::hash_object;

/// List the contents of a tree object with the specified hash
/// https://git-scm.com/docs/git-ls-tree
pub(crate) fn ls_tree(_name_only: bool, hash: &str) -> std::io::Result<()> {
    let mut tree = Object::from_hash(hash).expect("error here lol");
    assert_eq!(tree.header.object_type, ObjectType::TREE, "Object was not a tree");

    // Read the rest of the file
    let mut read_bytes = 0;
    while read_bytes < tree.header.size {
        let result = print_current_tree_entry(&mut tree).expect("error reading entry");
        read_bytes += result;
    }

    Ok(())
}

/// Prints the tree entry at the current position in the tree buffer reader
fn print_current_tree_entry(tree: &mut Object) -> Result<usize, &'static str> {
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
        hash: hex::encode(entry_buffer),
    };

    println!("{}", entry);

    Ok(read_bytes)
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
            if &path == &PathBuf::from("./.git") || &path == Path::new("./target") {
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

    println!("\n{}", tree_content);

    Ok(String::from("a directory"))
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
        write!(f, "{:0>6} {} {}    {}", self.mode as u32, self.object_type, self.hash, self.filename)
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