use crate::object;
use crate::object::tree::Tree;
use crate::object::{Hash, Object, ObjectType};
use std::ffi::CStr;
use std::io::{BufRead, Read};
use std::str::FromStr;

/// List the contents of a tree object with the specified hash
/// https://git-scm.com/docs/git-ls-tree
pub(crate) fn ls_tree(_name_only: bool, hash: &str) -> (std::io::Result<String>, Tree) {
    let mut tree = Object::from_hash(hash).expect("error here lol");
    assert_eq!(
        tree.header.object_type,
        ObjectType::TREE,
        "Object was not a tree"
    );

    // Read the rest of the file
    let mut read_bytes = 0;
    let mut result = String::new();
    let mut entries = Vec::new();
    while read_bytes < tree.header.size {
        let (entry, size) = get_current_tree_entry(&mut tree).expect("error reading entry");
        read_bytes += size;

        result.push_str(&format!("\n{}", entry.to_string().as_str()));
        entries.push(entry);
    }

    (Ok(result.trim_start().to_string()), Tree { entries })
}

/// Returns the tree entry at the current position in the tree buffer reader and its size in bytes
pub(crate) fn get_current_tree_entry(
    tree: &mut Object,
) -> Result<(object::tree::Entry, usize), &'static str> {
    let mut read_bytes = 0;

    let mut entry_buffer = Vec::new();
    tree.content_buffer_reader
        .read_until(b'\0', &mut entry_buffer)
        .expect("error reading header");
    read_bytes += entry_buffer.len();

    let header_string =
        CStr::from_bytes_with_nul(&entry_buffer).expect("File header missing null byte");
    let header_string = header_string
        .to_str()
        .expect("File header contains invalid UTF-8");

    let Some((mode, file_name)) = header_string.split_once(' ') else {
        panic! {"Entry missing space delimiter"}
    };
    let mode = mode.to_string();
    let file_name = file_name.to_string();

    entry_buffer.clear();
    entry_buffer.resize(20, 0);

    tree.content_buffer_reader
        .read_exact(&mut entry_buffer)
        .expect("error reading header");
    read_bytes += entry_buffer.len();

    let mode = object::tree::Mode::from_str(&mode).expect("Invalid mode");
    let entry = object::tree::Entry {
        mode,
        filename: file_name.to_string(),
        object_type: if mode == object::tree::Mode::DIRECTORY {
            ObjectType::TREE
        } else {
            ObjectType::BLOB
        },
        hash: Hash(entry_buffer),
    };

    Ok((entry, read_bytes))
}

#[cfg(test)]
mod tests {
    use crate::command::ls_tree::ls_tree;
    use crate::test_utils::{
        copy_git_object_file, run_git_command, setup_test_environment, teardown,
    };
    use rusty_fork::rusty_fork_test;
    use std::fs;
    use std::fs::File;
    use std::process::Command;

    rusty_fork_test! {
        #[test]
        fn ls_tree_test() {
            // Setup
            let repo = setup_test_environment().unwrap();

            let test_file_path = "test.txt";
            let _ = File::create(&test_file_path).unwrap();
            fs::write(&test_file_path, "this is some test content").unwrap();

            let test_dir_path = "testdir";
            fs::create_dir(&test_dir_path).unwrap();

            run_git_command(Command::new("git").arg("add").arg(".")).unwrap();
            let tree_hash = run_git_command(Command::new("git").arg("write-tree")).unwrap();

            copy_git_object_file(&tree_hash).unwrap();

            // Test
            let expected = run_git_command(Command::new("git").arg("ls-tree").arg(&tree_hash)).unwrap();
            let actual = ls_tree(false, &tree_hash).0.unwrap();

            assert_eq!(expected, actual);

            teardown(repo).unwrap();
        }
    }
}
