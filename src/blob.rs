use std::ffi::CStr;
use std::fs;
use std::fs::File;
use std::io::{BufRead, BufReader, Read, Write};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};

/// Reads the content of the git blob object stored in .git/objects with the specified hash
/// https://git-scm.com/docs/git-cat-file
pub(crate) fn cat_file(_pretty_print: bool, hash: &str) -> std::io::Result<()> {
    let subdirectory = &hash[..2];
    let file_name = &hash[2..];
    let file_path = format!(".git/objects/{}/{}", subdirectory, file_name);

    let compressed_file = File::open(file_path)?;

    let decompressor = ZlibDecoder::new(compressed_file);
    let mut file_buffer_reader = BufReader::new(decompressor);
    let mut file_buffer = Vec::new();

    // Read the header
    file_buffer_reader.read_until(b'\0', &mut file_buffer)?;
    let header = CStr::from_bytes_with_nul(&file_buffer).expect("File header missing null byte");
    let header = header.to_str().expect("File header contains invalid UTF-8");

    let Some((ty, size)) = header.split_once(' ') else { panic!("File header missing space delimiter") };
    assert_eq!(ty, "blob", "Object was not a blob");
    let size = size.parse::<usize>().expect("File header invalid size");
    
    file_buffer.clear();
    file_buffer.resize(size, 0);
    file_buffer_reader.read_to_end(&mut file_buffer)?;

    // Read the rest of the file
    let file_content = String::from_utf8(file_buffer).expect("File content is not valid UTF-8");
    
    println!("{}", &file_content);

    Ok(())
}

/// Generates a SHA1 hash for the specified file and writes its compressed version to the disk
/// if the w flag is used.
/// https://git-scm.com/docs/git-hash-object
pub(crate) fn hash_object(write: bool, file: &str) -> std::io::Result<()> {
    let uncompressed_file = File::open(file)?;
    let metadata = uncompressed_file.metadata()?;

    let header = format!("blob {}\0", metadata.len());

    // Compute the SHA1 hash
    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, &header);

    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    let reader = BufReader::new(uncompressed_file);
    for line in reader.lines() {
        let line = line?;

        Digest::update(&mut hasher, &line);

        // ZLib compression if the write flag is used
        if write {
            compressor.write_all(&line.as_bytes())?;
        }
    }

    let hash = hex::encode(hasher.finalize());

    println!("{}", &hash);

    // Write the compressed file to the disk if the write flag is used
    if write {
        let compressed_bytes = compressor.finish()?;

        let subdirectory = &hash[..2];
        let file_name = &hash[2..];
        let file_path = format!(".git/objects/{}/{}", subdirectory, file_name);

        println!("file_path: {}", file_path);

        fs::create_dir_all(format!(".git/objects/{}", subdirectory))?;
        let mut file = File::create(file_path)?;
        file.write_all(&compressed_bytes)?;
    }

    Ok(())
}