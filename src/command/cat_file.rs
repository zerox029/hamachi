use crate::object::Object;
use std::io::Read;

/// Reads the content of the git blob object stored in .git/objects with the specified hash
/// https://git-scm.com/docs/git-cat-file
pub fn cat_file(_pretty_print: bool, hash: &str) -> std::io::Result<String> {
    let mut blob = Object::from_hash(hash).expect("error here lol");
    assert_eq!(
        blob.header.object_type,
        crate::object::ObjectType::BLOB,
        "Object was not a blob"
    );

    // Read the rest of the file
    let mut content_buffer = Vec::new();
    blob.content_buffer_reader
        .read_to_end(&mut content_buffer)
        .expect("Couldn't read object file");
    let file_content = String::from_utf8(content_buffer).expect("File content is not valid UTF-8");

    Ok(file_content)
}

#[cfg(test)]
mod tests {
    use crate::test_utils::{
        copy_git_object_file, run_git_command, setup_test_environment, teardown,
    };
    use rusty_fork::rusty_fork_test;
    use std::fs;
    use std::fs::File;
    use std::process::Command;

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
}
