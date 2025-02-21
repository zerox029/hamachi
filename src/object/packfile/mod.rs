mod idx;

use std::fs;
use std::fs::File;
use std::io::{Read, Write};
use std::path::PathBuf;
use flate2::read::ZlibDecoder;
use sha1::{Digest, Sha1};
use crate::object::commit::Commit;
use crate::object::{Hash, Object};

#[derive(Debug)]
pub struct PackFile {
    header: PackFileHeader,
}

#[derive(Debug)]
pub struct PackFileHeader {
    pub hash: String,
    pub version: u32,
    pub entry_count: u32,
}

impl PackFile {
    pub fn new(data: Vec<u8>, hash: String) -> Self {
        // Write the file to disk
        fs::write(format!(".hamachi/objects/pack/pack-{}.pack", hash), &data).unwrap();

        let (header, mut read_bytes)= Self::parse_header(&data, hash);

        // Objects
        for _ in 0..header.entry_count {
            read_bytes += Self::parse_object(&data[read_bytes..]);
        }

        PackFile {
            header
        }
    }

    fn parse_header(data: &Vec<u8>, hash: String) -> (PackFileHeader, usize) {
        let mut read_pointer: usize = 0;

        println!("Packfile Headers");

        let _ = &data[read_pointer..read_pointer + 8];
        read_pointer += 8;

        // 4 byte signature PACK
        let signature = String::from_utf8(data[read_pointer..read_pointer + 4].to_vec()).unwrap();
        read_pointer += 4;
        println!("- Signature {}", signature);
        assert_eq!(signature, "PACK");

        // Version number
        let version = u32::from_be_bytes(data[read_pointer..read_pointer + 4].to_vec().try_into().unwrap());
        read_pointer += 4;
        println!("- Version {}", version);

        // Number of objects
        let entry_count = u32::from_be_bytes(data[read_pointer..read_pointer + 4].to_vec().try_into().unwrap());
        read_pointer += 4;
        println!("- Entry Count {}", entry_count);

        (PackFileHeader {
            hash,
            version,
            entry_count
        }, read_pointer)
    }

    fn parse_object(data: &[u8]) -> usize {
        let mut read_pointer = 0;

        // Parse the header
        let mut is_final_byte = *data.get(read_pointer).unwrap() < 128u8;
        let object_type = ObjectType::from_u8(*data.get(read_pointer).unwrap() >> 4 & 0b111).unwrap();
        read_pointer += 1;

        let mut size_bits = Vec::new();
        size_bits.push(*data.get(read_pointer).unwrap() & 0b1111);

        while !is_final_byte {
            let byte = *data.get(read_pointer).unwrap();

            is_final_byte = byte < 128u8;
            size_bits.push(byte & 0b1111111);
            read_pointer += 1;
        }

        // Parse the object data
        let compressed_size = match object_type {
            ObjectType::Commit => Self::handle_commit(&data[read_pointer..], 0),
            ObjectType::Blob => Self::handle_blob(&data[read_pointer..], 0),
            ObjectType::RefDelta => Self::handle_ref_delta(&data[read_pointer..], 0),
            _ => {
                println!("Unimplemented type {:?}", object_type);
                Self::handle_any(&data[read_pointer..], 0)
            }
        };

        read_pointer + compressed_size
    }

    fn handle_commit(data: &[u8], uncompressed_size: usize) -> usize {
        let (commit, read_bytes) = Commit::from_compressed_data(data);

        let commit_object_file_content = commit.to_object_file_representation();

        let mut hasher = Sha1::new();
        Digest::update(&mut hasher, &commit_object_file_content);
        let hash = Hash(hasher.finalize().to_vec()).to_string();

        let (subdirectory, file_name) = Object::get_path_from_hash(&hash).unwrap();
        let path = PathBuf::from(".hamachi/objects").join(subdirectory).join(file_name);
        if !fs::exists(&path).unwrap() {
            fs::create_dir_all(PathBuf::from(".hamachi/objects").join(subdirectory)).unwrap();
            let mut file = File::create(&path).unwrap();
            file.write(commit_object_file_content.as_slice()).unwrap();
        }

        read_bytes
    }

    fn handle_blob(data: &[u8], _uncompressed_size: usize) -> usize {
        println!("Reading blob");
        
        let mut decompressor = ZlibDecoder::new(data);
        let mut decompressed_data = String::new();
        decompressor.read_to_string(&mut decompressed_data).unwrap();


        decompressor.total_in() as usize
    }
    
    fn handle_ref_delta(data: &[u8], _uncompressed_size: usize) -> usize {
        println!("000\tOBJ_REF_DELTA\t30");
        
        let _hash = &data[..20];

        let mut decompressor = ZlibDecoder::new(&data[20..]);
        let mut decompressed_data = Vec::new();
        decompressor.read_to_end(&mut decompressed_data).unwrap();
        
        decompressor.total_in() as usize + 20
    }
    
    fn handle_any(data: &[u8], _uncompressed_size: usize) -> usize {
        let mut decompressor = ZlibDecoder::new(data);
        let mut decompressed_data = Vec::new();
        decompressor.read_to_end(&mut decompressed_data).unwrap();
        
        decompressor.total_in() as usize
    }
}

#[derive(Debug)]
enum ObjectType {
    Commit = 1,
    Tree = 2,
    Blob = 3,
    Tag = 4,
    OfsDelta = 6,
    RefDelta = 7,
}
impl ObjectType {
    fn from_u8(value: u8) -> Result<ObjectType, &'static str> {
        match value {
            1 => Ok(ObjectType::Commit),
            2 => Ok(ObjectType::Tree),
            3 => Ok(ObjectType::Blob),
            4 => Ok(ObjectType::Tag),
            6 => Ok(ObjectType::OfsDelta),
            7 => Ok(ObjectType::RefDelta),
            _ => Err("Unknown object type"),
        }
    }
}
