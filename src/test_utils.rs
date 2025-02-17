use std::{env, fs};
use std::io::Write;
use std::path::PathBuf;
use std::process::{Child, Command};
use crate::init;

/// Creates and sets working directory in a temporary directory and initializes a git and hamachi repo in it
pub fn setup_test_environment() -> std::io::Result<PathBuf> {
    // Create repo directory
    let temp_dir = env::temp_dir();
    let repo_name = format!("hamachi-{}", srfng::Generator::new().generate());
    let repo_path = PathBuf::from(temp_dir).join(repo_name);

    fs::create_dir(&repo_path)?;

    env::set_current_dir(&repo_path)?;

    // Create git repo
    Command::new("git").arg("init").output().expect("Failed to initialize git repo");
    Command::new("git").arg("config").arg("gc.auto").arg("0").output().expect("Failed to initialize git repo");
    Command::new("git").arg("config").arg("user.email").arg("osamu.dazai@gmail.com").output().expect("Failed to set user email");
    Command::new("git").arg("config").arg("user.name").arg("Osamu Dazai").output().expect("Failed to set user name");

    // Create hamachi repo
    init().expect("Failed to initialize hamachi repo");

    Ok(repo_path)
}

pub fn run_git_command(command: &mut Command) -> std::io::Result<String> {
    let output = command.output()?;
    let captured_stdout = String::from_utf8(output.stdout).expect("output is not valid UTF-8");

    Ok(captured_stdout.trim().to_string())
}

pub fn run_git_command_piped_input(mut command: Child, input: String) -> std::io::Result<String> {
    if let Some(mut stdin) = command.stdin.take() {
        stdin.write_all(input.as_bytes())?;
    }

    let output = command.wait_with_output()?;
    let captured_stdout = String::from_utf8(output.stdout).unwrap();
    
    Ok(captured_stdout.trim().to_string())
}

pub fn copy_git_object_file(hash: &str) -> std::io::Result<()> {
    let (subdirectory, file_name) = crate::object::Object::get_path_from_hash(hash).expect("Invalid hash");

    let from = PathBuf::from(".git/objects").join(subdirectory).join(file_name);
    let to = PathBuf::from(".hamachi/objects").join(subdirectory).join(file_name);

    let subdirectory = PathBuf::from(".hamachi/objects").join(&subdirectory);
    if !fs::exists(&subdirectory)? {
        fs::create_dir(&subdirectory)?;
    }
    
    fs::copy(from, to).expect("Couldn't copy object file");

    Ok(())
}

pub fn teardown(repo: PathBuf) -> std::io::Result<()> {
    env::set_current_dir("..")?;
    fs::remove_dir_all(&repo)?;

    Ok(())
}
