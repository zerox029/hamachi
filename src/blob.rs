use std::ffi::CStr;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};

pub(crate) fn cat_file(_pretty_print: bool, hash: &str) -> std::io::Result<()> {
    let subdirectory = &hash[..2];
    let file_name = &hash[2..];
    let file_path = format!(".git/objects/{}/{}", subdirectory, file_name);

    let compressed_file = File::open(file_path)?;

    let decompressor = ZlibDecoder::new(compressed_file);
    let mut file_buffer_reader = BufReader::new(decompressor);
    let mut file_buffer = Vec::new();
    
    file_buffer_reader.read_until(b'\0', &mut file_buffer)?;
    let header = CStr::from_bytes_with_nul(&file_buffer).expect("File header missing null byte");
    let header = header.to_str().expect("File header contains invalid UTF-8");

    let size = header.split_once(' ').expect("File header missing space delimiter").1;
    let size = size.parse::<usize>().expect("File header invalid size");
    
    file_buffer.clear();
    file_buffer.resize(size, 0);
    file_buffer_reader.read_to_end(&mut file_buffer)?;
    
    let file_content = String::from_utf8(file_buffer).expect("File content is not valid UTF-8");
    
    println!("{}", &file_content);

    Ok(())
}

pub(crate) fn hash_object(write: bool, file: &str) {
    let uncompressed_file_content = fs::read(&file).unwrap();

    let header = format!("blob {}", uncompressed_file_content.len());
    let blob = format!("{}\0{}", header, String::from_utf8(uncompressed_file_content).unwrap());

    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, &blob);
    let hash = hex::encode(hasher.finalize());

    println!("{}", hash);

    if write {
        let mut compresser = ZlibEncoder::new(Vec::new(), Compression::default());
        compresser.write_all(&blob.as_bytes()).unwrap();
        let compressed_bytes = compresser.finish().unwrap();

        let subdirectory = &hash[..2];
        let file_name = &hash[2..];
        let file_path = format!(".git/objects/{}/{}", subdirectory, file_name);

        println!("file_path: {}", file_path);

        fs::create_dir_all(format!(".git/objects/{}", subdirectory)).unwrap();
        let mut file = File::create(file_path).unwrap();
        file.write_all(&compressed_bytes).unwrap();
    }
}