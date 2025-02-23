use crate::object::{Hash, Object, ObjectType};
use std::io::Read;

pub fn parse_ref_delta_file(data: Vec<u8>, base_hash: &Hash) -> Vec<u8> {
    let mut read_pointer = 0;

    // Base object
    let mut base_object = Object::from_hash(&base_hash.to_string()).unwrap();
    let mut base_object_content = Vec::new();
    base_object
        .content_buffer_reader
        .read_to_end(&mut base_object_content)
        .expect("TODO: panic message");

    // Parse source size
    let (source_size, read_bytes) = parse_varint(&data[read_pointer..]);
    read_pointer += read_bytes;

    assert_eq!(base_object_content.len(), source_size);

    // Parse target source size
    let (target_size, read_bytes) = parse_varint(&data[read_pointer..]);
    read_pointer += read_bytes;

    let mut undeltified_data = Vec::with_capacity(target_size);
    while read_pointer < data.len() {
        let (mut parsed_data, read_bytes) =
            parse_instruction(&data[read_pointer..], &base_object_content);
        read_pointer += read_bytes;

        undeltified_data.append(&mut parsed_data);
    }

    // TODO: Move this out of here
    match base_object.header.object_type {
        ObjectType::BLOB => {
            todo!()
        }
        ObjectType::TREE => {
            let mut header = format!("tree {}\0", target_size).as_bytes().to_vec();
            header.append(&mut undeltified_data);

            header
        }
        ObjectType::COMMIT => {
            todo!()
        }
    }
}

fn parse_varint(data: &[u8]) -> (usize, usize) {
    let mut byte = 0x80;
    let mut val = 0usize;
    let mut shift = 0usize;

    let mut read_bytes = 0;

    while (byte & 0x80) > 0 {
        byte = *data.get(read_bytes).unwrap();
        val += ((byte & 127) as usize) << shift;
        shift += 7;

        read_bytes += 1;
    }

    (val, read_bytes)
}

fn parse_variable_size_integer(data: &[u8]) -> Vec<u8> {
    let mut read_pointer = 0;
    let mut is_final_byte = false;
    let mut source_size_bits = Vec::new();

    while !is_final_byte {
        let byte = data.get(read_pointer).unwrap();

        is_final_byte = *byte < 128u8;
        source_size_bits.push(byte & 0b1111111);
        read_pointer += 1;
    }

    source_size_bits
}

fn parse_instruction(data: &[u8], base_object: &Vec<u8>) -> (Vec<u8>, usize) {
    let msb = data.get(0).unwrap() >> 7;
    match msb {
        0b0 => {
            let parsed_data = parse_insert_instruction(data);
            let len = parsed_data.len() + 1;
            (parsed_data, len)
        }
        0b1 => parse_copy_instruction(data, base_object),
        _ => {
            panic!(
                "Something catastrophic happened the universe is probably on the verge of collapse"
            )
        }
    }
}

fn parse_copy_instruction(data: &[u8], base_object: &Vec<u8>) -> (Vec<u8>, usize) {
    let mut read_pointer = 1;

    let mut offset_bytes: [u8; 4] = [0, 0, 0, 0];
    for i in 0..4 {
        if data.get(0).unwrap() & (2u8.pow(i)) > 0 {
            offset_bytes[i as usize] = *data.get(read_pointer).unwrap();
            read_pointer += 1;
        }
    }

    let mut length_bytes: [u8; 4] = [0, 0, 0, 0];
    for i in 0..3 {
        if ((data.get(0).unwrap() >> 4) & (2u8.pow(2 - i))) > 0u8 {
            length_bytes[i as usize + 1] = *data.get(read_pointer).unwrap();
            read_pointer += 1;
        }
    }

    let offset = u32::from_be_bytes(offset_bytes);
    let length = u32::from_be_bytes(length_bytes);

    let copy_content = &base_object[offset as usize..(offset + length) as usize];

    (copy_content.to_vec(), read_pointer)
}

fn parse_insert_instruction(data: &[u8]) -> Vec<u8> {
    let size = data.get(0).unwrap() & 0b1111111;

    data[1..size as usize + 1].to_vec()
}
