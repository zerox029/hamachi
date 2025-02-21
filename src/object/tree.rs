use std::cmp::PartialEq;
use std::fmt::{Display};
use std::io::Read;
use std::str::FromStr;
use flate2::read::ZlibDecoder;
use sha1::{Digest, Sha1};
use crate::object::{Hash, ObjectType};

pub(crate) struct Tree {
    entries: Vec<Entry>,
}

impl Tree {
    pub(crate) fn from_packfile_compressed_data(data: &[u8]) -> (Self, usize) {
        let mut decompressor = ZlibDecoder::new(data);
        let mut decompressed_data = Vec::new();
        decompressor.read_to_end(&mut decompressed_data).unwrap();
        
        let mut entries = Vec::new();
        let mut read_pointer = 0;

        while read_pointer < decompressed_data.len() {
            let null_separator = decompressed_data[read_pointer..].iter().position(|&c| c == b'\0').unwrap();

            let mode_name = &decompressed_data[read_pointer..read_pointer + null_separator];
            let mode_name_str = String::from_utf8(mode_name.to_vec()).unwrap();
            let mut parts = mode_name_str.split_whitespace();

            let mode = Mode::from_str(parts.next().unwrap()).unwrap();
            let filename = parts.next().unwrap().to_string();
            
            let object_type = ObjectType::from_file_mode(mode);

            read_pointer += null_separator + 1;

            let hash = Hash(decompressed_data[read_pointer..read_pointer + 20].to_vec());

            read_pointer += 20;

            entries.push(Entry {
                mode, filename, object_type, hash
            })
        }

        (Tree { entries }, decompressor.total_in() as usize)
    }
    
    pub(crate) fn generate_object_file_representation(&mut self) -> Vec<u8> {
        let mut entry_byte_vectors = Vec::new();
        for entry in &mut self.entries {
            let mut entry_bytes = format!("{} {}\0", entry.mode as u32, entry.filename).as_bytes().to_vec();
            entry_bytes.append(&mut entry.hash.0);

            entry_byte_vectors.push(entry_bytes);
        }

        let entries_section = entry_byte_vectors.into_iter().flatten().collect::<Vec<u8>>();
        let header = format!("tree {}\0", entries_section.len()).as_bytes().to_vec();
        
        vec![header, entries_section].concat()
    }
}

#[derive(Debug)]
pub(crate) struct Entry {
    pub(crate) mode: Mode,
    pub(crate) filename: String,
    pub(crate) object_type: ObjectType,
    pub(crate) hash: Hash,
}

impl Display for Entry {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:0>6} {} {}\t{}", self.mode as u32, self.object_type, &self.hash.to_string(), self.filename)
    }
}

#[derive(Clone, Copy, PartialOrd, PartialEq, Debug)]
pub(crate) enum Mode {
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
impl Mode {
    fn from_bytes(bytes: &[u8]) -> Result<Self, ()> {
        let string = String::from_utf8(bytes.to_vec()).map_err(|_| ())?;

        Self::from_str(string.as_str())
    }
}

