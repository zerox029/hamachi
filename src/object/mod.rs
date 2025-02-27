use crate::object::tree::Mode;
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fmt::{Display, Formatter};
use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub mod blob;
pub mod commit;
pub mod packfile;
pub mod tree;

#[derive(Debug)]
pub struct Object {
    pub header: Header,
    pub content_buffer_reader: BufReader<ZlibDecoder<File>>,
}

#[derive(Debug)]
pub struct Header {
    pub(crate) object_type: ObjectType,
    pub(crate) size: usize,
}

#[derive(Debug, PartialOrd, Eq, PartialEq)]
pub(crate) enum ObjectType {
    BLOB,
    TREE,
    COMMIT,
}

impl Object {
    pub(crate) fn from_hash(hash: &str) -> Result<Object, &'static str> {
        let (subdirectory, file_name) = Self::get_path_from_hash(hash).expect("Invalid hash");
        let file_path = format!(".hamachi/objects/{}/{}", subdirectory, file_name);

        let compressed_file = File::open(file_path).expect("Can't open object file");

        let decompressor = ZlibDecoder::new(compressed_file);
        let mut file_buffer_reader = BufReader::new(decompressor);
        let mut file_buffer = Vec::new();

        // Read the header
        file_buffer_reader
            .read_until(b'\0', &mut file_buffer)
            .expect("Couldn't read object file");
        let header_string =
            CStr::from_bytes_with_nul(&file_buffer).expect("File header missing null byte");
        let header_string = header_string
            .to_str()
            .expect("File header contains invalid UTF-8");

        let Some((ty, size)) = header_string.split_once(' ') else {
            panic!("File header missing space delimiter")
        };
        let size = size.parse::<usize>().expect("File header invalid size");

        let header = Header {
            object_type: ObjectType::from_str(ty).expect("Invalid file type"),
            size,
        };

        file_buffer.clear();
        file_buffer.resize(size, 0);

        Ok(Object {
            header,
            content_buffer_reader: file_buffer_reader,
        })
    }

    pub(crate) fn write_to_disk(hash: &Hash, content: &Vec<u8>) -> std::io::Result<()> {
        let string_hash = hash.to_string();
        let (subdirectory, file_name) =
            Self::get_path_from_hash(&string_hash).expect("Invalid hash");
        let file_path = &format!(".hamachi/objects/{}/{}", subdirectory, file_name);
        let file_path = Path::new(file_path);

        fs::create_dir_all(format!(".hamachi/objects/{}", subdirectory))?;

        if file_path.exists() {
            let mut perms = fs::metadata(&file_path)?.permissions();
            perms.set_readonly(false);
            fs::set_permissions(&file_path, perms)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(file_path)?;

        file.write_all(&content)?;

        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file_path, perms)?;

        Ok(())
    }

    pub fn get_path_from_hash(hash: &str) -> std::io::Result<(&str, &str)> {
        let subdirectory = &hash[..2];
        let file_name = &hash[2..];

        Ok((subdirectory, file_name))
    }

    pub fn decompress_object(hash: &str, is_git: bool) -> std::io::Result<Vec<u8>> {
        let (subdirectory, file_name) = Self::get_path_from_hash(hash).expect("Invalid hash");
        let path = PathBuf::from(if is_git {
            ".git/objects"
        } else {
            ".hamachi/objects"
        })
        .join(&subdirectory)
        .join(&file_name);
        let file = File::open(path)?;

        let mut decompressed = Vec::new();

        let mut decompressor = ZlibDecoder::new(file);
        decompressor.read_to_end(&mut decompressed)?;

        Ok(decompressed)
    }
}

impl ObjectType {
    pub fn from_file_mode(mode: Mode) -> Self {
        match mode {
            Mode::DIRECTORY => Self::TREE,
            _ => Self::BLOB,
        }
    }
}
impl FromStr for ObjectType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "blob" => Ok(ObjectType::BLOB),
            "tree" => Ok(ObjectType::TREE),
            _ => Ok(ObjectType::TREE),
        }
    }
}

impl Display for ObjectType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                ObjectType::BLOB => "blob",
                ObjectType::TREE => "tree",
                ObjectType::COMMIT => "commit",
            }
        )
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Hash(pub Vec<u8>);

impl Display for Hash {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", hex::encode(&self.0))
    }
}

impl FromStr for Hash {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        hex::decode(s).map_err(|_| ()).map(|v| Hash(v))
    }
}
