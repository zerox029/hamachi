use std::{env, fs};
use std::io::Read;
use flate2::read::ZlibDecoder;

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];
    
    match command.as_str() {
        "cat-file" => {
            cat_file(&args[2])
        }
        _ => {}
    }
}

fn cat_file(hash: &str) {
    let subdirectory = &hash[..2];
    let file_name = &hash[2..];
    
    let file_path = format!(".git/objects/{}/{}", subdirectory, file_name);
    let compressed_file_content = fs::read(&file_path).unwrap();
    let compressed_file_content = compressed_file_content.as_slice();
    
    let mut decompresser = ZlibDecoder::new(compressed_file_content);
    let mut decompressed_file_content = String::new();
    decompresser.read_to_string(&mut decompressed_file_content).unwrap();

    let components = decompressed_file_content.split('\0').collect::<Vec<&str>>();
    let header = components[0].split(" ").collect::<Vec<&str>>();

    if header[0] != "blob" {
        panic!("Not a blob")
    }

    let blob = Blob{size: header[1].parse::<usize>().unwrap(), data: components[1].to_string()};

    println!("size: {}", blob.size);
    println!("data: {}", blob.data)
}

struct Blob {
    size: usize,
    data: String,
}