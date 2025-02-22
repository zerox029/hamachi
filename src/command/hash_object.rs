use crate::object::{Hash, Object};
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs::File;
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;

/// Generates a SHA1 hash for the specified file and writes its compressed version to the disk
/// if the w flag is used.
/// https://git-scm.com/docs/git-hash-object
pub(crate) fn hash_object(write: bool, file: &PathBuf) -> std::io::Result<Hash> {
    let uncompressed_file = File::open(file).expect("Couldn't open file");
    let metadata = uncompressed_file.metadata()?;

    let header = format!("blob {}\0", metadata.len());

    // Compute the SHA1 hash
    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, &header);

    let mut compressor = ZlibEncoder::new(Vec::new(), Compression::default());
    if write {
        compressor.write(&header.as_bytes())?;
    }

    let reader = BufReader::new(uncompressed_file);
    for line in reader.lines() {
        let line = line?;

        Digest::update(&mut hasher, &line);

        // ZLib compression if the write flag is used
        if write {
            compressor.write(&line.as_bytes())?;
        }
    }

    let hash: Hash = Hash(hasher.finalize().as_slice().to_vec());

    // Write the compressed file to the disk if the write flag is used
    if write {
        let compressed_bytes = compressor.finish()?;

        Object::write_to_disk(&hash, &compressed_bytes)?;
    }

    Ok(hash)
}

#[cfg(test)]
mod tests {
    use crate::command::hash_object::hash_object;
    use crate::object::Object;
    use crate::test_utils::{run_git_command, setup_test_environment, teardown};
    use flate2::read::ZlibDecoder;
    use rusty_fork::rusty_fork_test;
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    rusty_fork_test! {
       #[test]
        fn hash_object_no_write() {
            // Setup
            let repo = setup_test_environment().unwrap();

            let test_file_path = "test_file.txt";
            File::create(test_file_path).unwrap();

            // Test
            let expected = run_git_command(Command::new("git").arg("hash-object").arg(test_file_path)).unwrap();
            let actual = hash_object(false, &PathBuf::from(test_file_path)).unwrap().to_string();

            assert_eq!(expected, actual);
            assert!(Path::new(".hamachi/objects").read_dir().unwrap().next().is_none());

            teardown(repo).unwrap();
        }
    }

    rusty_fork_test! {
       #[test]
        fn hash_object_write() {
            // Setup
            let repo = setup_test_environment().unwrap();

            let test_file_name = "test_file.txt";
            let test_file_content = "this is some test content";
            File::create(test_file_name).unwrap();
            fs::write(test_file_name, test_file_content).unwrap();

            // Test
            let expected_hash = run_git_command(Command::new("git").arg("hash-object").arg("-w").arg(test_file_name)).unwrap();
            let actual_hash = hash_object(true, &PathBuf::from(test_file_name)).unwrap().to_string();

            let (subdirectory, file_name) = Object::get_path_from_hash(&actual_hash).unwrap();
            let expected_file = File::open(PathBuf::from(".git/objects").join(subdirectory).join(file_name)).expect("Git object file not found");
            let actual_file = File::open(PathBuf::from(".hamachi/objects").join(subdirectory).join(file_name)).expect("Hamachi object file not found");
            let mut expected_file_content = String::new();
            let mut actual_file_content = String::new();
            ZlibDecoder::new(expected_file).read_to_string(&mut expected_file_content).unwrap();
            ZlibDecoder::new(actual_file).read_to_string(&mut actual_file_content).unwrap();

            assert_eq!(expected_hash, actual_hash);
            assert_eq!(expected_file_content, actual_file_content);

            teardown(repo).unwrap()
        }
    }
}
