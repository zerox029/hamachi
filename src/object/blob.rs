use std::fs;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use flate2::Compression;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};
use crate::object::{Object, ObjectType};

/// Reads the content of the git blob object stored in .git/objects with the specified hash
/// https://git-scm.com/docs/git-cat-file
pub(crate) fn cat_file(_pretty_print: bool, hash: &str) -> std::io::Result<()> {
    let mut blob = Object::from_hash(hash).expect("error here lol");
    assert_eq!(blob.header.object_type, ObjectType::BLOB, "Object was not a blob");

    // Read the rest of the file
    let mut content_buffer = Vec::new();
    blob.content_buffer_reader.read_to_end(&mut content_buffer).expect("Couldn't read object file");
    let file_content = String::from_utf8(content_buffer).expect("File content is not valid UTF-8");

    println!("{}", &file_content);

    Ok(())
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

        let subdirectory = &hash[..2];
        let file_name = &hash[2..];
        let file_path = &format!(".git/objects/{}/{}", subdirectory, file_name);
        let file_path = Path::new(file_path);

        fs::create_dir_all(format!(".git/objects/{}", subdirectory))?;

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

        file.write_all(&compressed_bytes)?;

        let mut perms = fs::metadata(&file_path)?.permissions();
        perms.set_readonly(true);
        fs::set_permissions(&file_path, perms)?;
    }

    Ok(hash)
}