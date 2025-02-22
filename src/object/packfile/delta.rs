use crate::object::Hash;
use std::ptr::read;

pub fn parse_ref_delta_file(data: Vec<u8>, base_hash: &Hash) {
    let mut read_pointer = 0;
    
    println!("{}", base_hash);

    // Parse source size
    let source_size = parse_variable_size_integer(&data[read_pointer..]);
    read_pointer += source_size.len();

    // Parse target source size
    let target_size = parse_variable_size_integer(&data[read_pointer..]);
    read_pointer += target_size.len();

    let parsed_data = parse_instruction(&data[read_pointer..], base_hash);
    unsafe { println!("{}", String::from_utf8_unchecked(parsed_data)) };
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

fn parse_instruction(data: &[u8], base_hash: &Hash) -> Vec<u8> {
    let msb = data.get(0).unwrap() >> 7;
    match msb {
        0b0 => {
            parse_copy_instruction(data, base_hash)
        }
        0b1 => {
            parse_insert_instruction(data)
        }
        _ => {
            panic!(
                "Something catastrophic happened the universe is probably on the verge of collapse"
            )
        }
    }
}

fn parse_copy_instruction(data: &[u8], base_hash: &Hash) -> Vec<u8> {
    let mut read_pointer = 1;

    let mut offset_bytes: [u8; 4] = [0, 0, 0, 0];
    for i in 0..4 {
        if data.get(0).unwrap() & (2u8.pow(i)) > 0 {
            offset_bytes[i as usize] = *data.get(read_pointer as usize).unwrap();
            read_pointer += 1;
        }
    }
    read_pointer += 1;

    let mut length_bytes: [u8; 4] = [0, 0, 0, 0];
    for i in 0..3 {
        if ((data.get(0).unwrap() >> 4) & (2u8.pow(2 - i))) > 0u8 {
            length_bytes[i as usize] = *data.get(i as usize).unwrap();
            read_pointer += 1;
        }
    }
    read_pointer += 1;

    let offset = u32::from_le_bytes(offset_bytes);
    let length = u32::from_le_bytes(length_bytes);

    todo!()
}

fn parse_insert_instruction(data: &[u8]) -> Vec<u8> {
    let size = data.get(0).unwrap() & 0b1111111;
    
    data[1..size as usize + 1].to_vec()
}
