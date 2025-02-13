use std::cmp::PartialEq;
use std::ffi::CStr;
use std::fmt::Display;
use std::io::{BufRead, Read};
use std::str::FromStr;
use crate::object::{Object, ObjectType};

pub(crate) fn ls_tree(_name_only: bool, hash: &str) -> std::io::Result<()> {
    let mut tree = Object::from_hash(hash).expect("error here lol");
    assert_eq!(tree.header.object_type, ObjectType::TREE, "Object was not a tree");

    // Read the rest of the file
    let mut read_bytes = 0;
    while read_bytes < tree.header.size {
        let result = read_tree_entry(&mut tree).expect("error reading entry");
        read_bytes += result;
    }


    Ok(())
}
fn read_tree_entry(tree: &mut Object) -> Result<usize, &'static str> {
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

#[derive(Clone, Copy, PartialOrd, PartialEq)]
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