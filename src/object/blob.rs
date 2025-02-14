use std::fs::{File};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{PathBuf};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use crate::object::{Object, ObjectType};

/// Reads the content of the git blob object stored in .git/objects with the specified hash
/// https://git-scm.com/docs/git-cat-file
pub(crate) fn cat_file(_pretty_print: bool, hash: &str) -> std::io::Result<String> {
    let mut blob = Object::from_hash(hash).expect("error here lol");
    assert_eq!(blob.header.object_type, ObjectType::BLOB, "Object was not a blob");

    // Read the rest of the file
    let mut content_buffer = Vec::new();
    blob.content_buffer_reader.read_to_end(&mut content_buffer).expect("Couldn't read object file");
    let file_content = String::from_utf8(content_buffer).expect("File content is not valid UTF-8");

    Ok(file_content)
}

/// Generates a SHA1 hash for the specified file and writes its compressed version to the disk
/// if the w flag is used.
/// https://git-scm.com/docs/git-hash-object
pub(crate) fn hash_object(write: bool, file: &PathBuf) -> std::io::Result<String> {
    let uncompressed_file = File::open(file).expect("Couldn't open file");
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

    // Write the compressed file to the disk if the write flag is used
    if write {
        let compressed_bytes = compressor.finish()?;

        Object::write_to_disk(&hash, &compressed_bytes)?;
    }

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::process::Command;
    use super::*;
    #[test]
    fn cat_file() {
        // Setup
        let test_file_path = "test_file.txt";
        let _ = File::create(test_file_path);
        fs::write(test_file_path, "this is some test content").unwrap();

        let hash = Command::new("git").arg("hash-object").arg("-w").arg(test_file_path).output().unwrap();
        let hash = String::from_utf8(hash.stdout).unwrap();
        let hash = hash.trim();

        let subdirectory = &hash[..2];
        let file_name = &hash[2..];
        
        let from = format!(".git/objects/{}/{}", subdirectory, file_name.trim());
        let to = format!(".hamachi/objects/{}/{}", subdirectory, file_name.trim());
        fs::copy(from, to).unwrap();
        
        fs::copy(format!(".git/objects/{}/{}", subdirectory, file_name),
                 format!(".hamachi/objects/{}/{}", subdirectory, file_name))
            .unwrap();
         
        // Test
        let expected = Command::new("git").arg("cat-file").arg(test_file_path).output().unwrap();
        let expected = String::from_utf8(expected.stdout).unwrap();
        
        let actual = super::cat_file(false, &hash).unwrap();
        
        assert_eq!(expected, actual);
    }
}