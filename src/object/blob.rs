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
    use std::{env, fs};
    use std::path::Path;
    use std::process::Command;
    use file_diff::diff_files;
    use flate2::read::ZlibDecoder;
    use rusty_fork::rusty_fork_test;
    use crate::init;
    use super::*;

    /// Creates and sets working directory in a temporary directory and initializes a git and hamachi repo in it
    fn setup_test_environment() -> std::io::Result<PathBuf> {
        // Create repo directory
        let temp_dir = env::temp_dir();
        let repo_name = format!("hamachi-{}", srfng::Generator::new().generate());
        let repo_path = PathBuf::from(temp_dir).join(repo_name);

        fs::create_dir(&repo_path)?;

        env::set_current_dir(&repo_path)?;

        // Create git repo
        Command::new("git").arg("init").output().expect("Failed to initialize git repo");
        Command::new("git").arg("config").arg("gc.auto").arg("0").output().expect("Failed to disable git garbage collection");

        // Create hamachi repo
        init().expect("Failed to initialize hamachi repo");

        Ok(repo_path)
    }

    fn run_git_command(command: &mut Command) -> std::io::Result<String> {
        let output = command.output()?;
        let captured_stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");

        Ok(captured_stdout.trim().to_string())
    }

    fn copy_git_object_file(hash: &str) -> std::io::Result<()> {
        let (subdirectory, file_name) = Object::get_path_from_hash(hash).expect("Invalid hash");

        let from = PathBuf::from(".git/objects").join(subdirectory).join(file_name);
        let to = PathBuf::from(".hamachi/objects").join(subdirectory).join(file_name);

        let subdirectory = PathBuf::from(".hamachi/objects").join(&subdirectory);
        if !fs::exists(&subdirectory)? {
            fs::create_dir(&subdirectory)?;
        }
        
        fs::copy(from, to).expect("Couldn't copy object file");

        Ok(())
    }

    fn teardown(repo: PathBuf) -> std::io::Result<()> {
        env::set_current_dir("..")?;
        fs::remove_dir_all(&repo)?;

        Ok(())
    }

    rusty_fork_test! {
        #[test]
        fn cat_file() {
            // Setup
            let repo = setup_test_environment().unwrap();

            let test_file_path = "test_file.txt";
            let _ = File::create(test_file_path).unwrap();
            fs::write(test_file_path, "this is some test content").unwrap();

            let hash = run_git_command(Command::new("git").arg("hash-object").arg("-w").arg(test_file_path))
                .expect("Failed to hash object");

            copy_git_object_file(&hash).unwrap();

            // Test
            let expected = run_git_command(Command::new("git").arg("cat-file").arg("blob").arg(&hash))
                .expect("Failed to cat file");
            let actual = super::cat_file(false, &hash).unwrap();

            assert_eq!(expected, actual);

            teardown(repo).unwrap();
        }
    }

    rusty_fork_test! {
       #[test]
        fn cat_empty_file() {
            // Setup
            let repo = setup_test_environment().unwrap();

            let test_file_path = "test_file.txt";
            File::create(test_file_path).unwrap();

            let hash = run_git_command(Command::new("git").arg("hash-object").arg("-w").arg(test_file_path))
                .expect("Failed to hash object");

            copy_git_object_file(&hash).unwrap();

            // Test
            let expected = run_git_command(Command::new("git").arg("cat-file").arg("blob").arg(&hash))
                .expect("Failed to cat file");
            let actual = super::cat_file(false, &hash).unwrap();

            assert_eq!(expected, actual);

            teardown(repo).unwrap();
        }
    }

    rusty_fork_test! {
       #[test]
        fn hash_object_no_write() {
            // Setup
            let repo = setup_test_environment().unwrap();

            let test_file_path = "test_file.txt";
            File::create(test_file_path).unwrap();

            // Test
            let expected = run_git_command(Command::new("git").arg("hash-object").arg(test_file_path)).unwrap();
            let actual = hash_object(false, &PathBuf::from(test_file_path)).unwrap();

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
            let actual_hash = hash_object(true, &PathBuf::from(test_file_name)).unwrap();

            let (subdirectory, file_name) = Object::get_path_from_hash(&actual_hash).unwrap();
            let expected_file = File::open(PathBuf::from(".git/objects").join(subdirectory).join(file_name)).expect("Git object file not found");
            let actual_file = File::open(PathBuf::from(".hamachi/objects").join(subdirectory).join(file_name)).expect("Hamachi object file not found");
            let mut expected_file_content = String::new();
            let mut actual_file_content = String::new();
            ZlibDecoder::new(expected_file).read_to_string(&mut expected_file_content).unwrap();
            ZlibDecoder::new(actual_file).read_to_string(&mut actual_file_content).unwrap();

            assert_eq!(expected_hash, actual_hash);
            assert_eq!(expected_file_content, actual_file_content);
        }
    }
}