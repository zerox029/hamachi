use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::str::FromStr;
use flate2::read::ZlibDecoder;

pub mod blob;
pub mod tree;

#[derive(Debug)]
struct Object {
    header: Header,
    content_buffer_reader: BufReader<ZlibDecoder<File>>,
}

#[derive(Debug)]
struct Header {
    object_type: ObjectType,
    size: usize,
}

#[derive(Debug, PartialOrd, PartialEq)]
enum ObjectType {
    BLOB,
    TREE,
    COMMIT,
}

impl Object {
    fn from_hash(hash: &str) -> Result<Object, &'static str> {
        let subdirectory = &hash[..2];
        let file_name = &hash[2..];
        let file_path = format!(".git/objects/{}/{}", subdirectory, file_name);

        let compressed_file = File::open(file_path).expect("Can't open object file");

        let decompressor = ZlibDecoder::new(compressed_file);
        let mut file_buffer_reader = BufReader::new(decompressor);
        let mut file_buffer = Vec::new();

        // Read the header
        file_buffer_reader.read_until(b'\0', &mut file_buffer).expect("Couldn't read object file");
        let header_string = CStr::from_bytes_with_nul(&file_buffer).expect("File header missing null byte");
        let header_string = header_string.to_str().expect("File header contains invalid UTF-8");

        let Some((ty, size)) = header_string.split_once(' ') else { panic!("File header missing space delimiter") };
        let size = size.parse::<usize>().expect("File header invalid size");

        let header = Header{ object_type: ObjectType::from_str(ty).expect("Invalid file type"), size };

        file_buffer.clear();
        file_buffer.resize(size, 0);

        Ok(Object{header, content_buffer_reader: file_buffer_reader})
    }
}

impl FromStr for ObjectType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => { Ok(ObjectType::BLOB) },
            "tree" => { Ok(ObjectType::TREE) },
            _ =>  { Ok(ObjectType::TREE) }
        }
    }
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f,"{}", match self {
            ObjectType::BLOB => "blob",
            ObjectType::TREE => "tree",
            ObjectType::COMMIT => "commit",
            _ => "unknown"
        })
    }
}