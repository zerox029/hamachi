use std::fs;
use std::io::Read;
use std::ptr::read;
use flate2::read::ZlibDecoder;
use reqwest::blocking::Response;

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

        let (header, read_bytes)= Self::parse_header(&data, hash);

        println!("{:#?}", &header);

        // Objects
        let read_bytes = Self::parse_object(&data[read_bytes..]);
        println!("{}", read_bytes);
        
        let read_bytes = Self::parse_object(&data[read_bytes..]);

        PackFile {
            header
        }
    }

    fn parse_header(data: &Vec<u8>, hash: String) -> (PackFileHeader, usize) {
        let mut read_pointer: usize = 0;

        let unknown_header_thing = &data[read_pointer..read_pointer + 8];
        read_pointer += 8;

        // 4 byte signature PACK
        let signature = &data[read_pointer..read_pointer + 4];
        read_pointer += 4;

        // Version number
        let version = u32::from_be_bytes(data[read_pointer..read_pointer + 4].to_vec().try_into().unwrap());
        read_pointer += 4;

        // Number of objects
        let entry_count = u32::from_be_bytes(data[read_pointer..read_pointer + 4].to_vec().try_into().unwrap());
        read_pointer += 4;

        (PackFileHeader {
            hash,
            version,
            entry_count
        }, read_pointer)
    }

    fn parse_object(data: &[u8]) -> usize {
        let mut read_pointer = 0;

        let mut is_final_byte = *data.get(read_pointer).unwrap() < 128u8;
        let object_type = ObjectType::from_u8(*data.get(read_pointer).unwrap() >> 4 & 0b111).unwrap();
        read_pointer += 1;

        let mut size_bits = Vec::new();
        while !is_final_byte {
            let byte = *data.get(read_pointer).unwrap();

            is_final_byte = byte < 128u8;
            size_bits.push(byte & 0b1111111);
            read_pointer += 1;
        }

        let mut decompressor = ZlibDecoder::new(&data[read_pointer..]);
        let mut decompressed_data = String::new();
        decompressor.read_to_string(&mut decompressed_data).unwrap();

        println!("Object content: \n{}", decompressed_data);

        decompressor.total_in() as usize
    }

    fn parse_object_header(response: &mut Response) -> (ObjectType, Vec<u8>) {
        let mut data = Vec::new();
        data.resize(1, 0);
        response.read_exact(&mut data).unwrap();

        let mut is_final_byte = *data.get(0).unwrap() < 128u8;
        let object_type = ObjectType::from_u8(*data.get(0).unwrap() >> 4 & 0b111).unwrap();
        let mut size_bits = Vec::new();
        size_bits.push(*data.get(0).unwrap() & 0b1111);

        while !is_final_byte {
            data.clear();
            data.resize(1, 0);
            response.read_exact(&mut data).unwrap();

            is_final_byte = *data.get(0).unwrap() < 128u8;
            size_bits.push(*data.get(0).unwrap() & 0b1111111);
        }

        (object_type, size_bits)
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
