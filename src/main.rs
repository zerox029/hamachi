mod cli;
mod object;
mod test_utils;

use std::fs;
use std::path::{Path, PathBuf};
use clap::Parser;
use cli::{Args, Command};
use object::blob::{cat_file, hash_object};
use object::tree::ls_tree;
use object::tree::write_tree;
use object::commit::commit_tree;

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            let _ = init();
        },
        Command::CatFile { pretty_print, hash } => {
            let file_content = cat_file(pretty_print, &hash).unwrap();

            println!("{:?}", file_content)
        },
        Command::HashObject { write, file } => {
            let hash = hash_object(write, &PathBuf::from(file)).unwrap().to_string();

            println!("{hash}")
        },
        Command::LsTree { name_only, hash } => {
            let tree_content = ls_tree(name_only, &hash).unwrap();
            
            println!("{tree_content}")
        },
        Command::WriteTree => {
            let tree_hash = write_tree(None).unwrap().to_string();
            
            println!("{tree_hash}");
        },
        Command::CommitTree { hash, message } => {
            let commit_hash = commit_tree(hash, message).unwrap().to_string();
            
            println!("{commit_hash}");
        }
    }
}

/// Initialize a new git repository
/// https://git-scm.com/docs/git-init
fn init() -> std::io::Result<()> {
    fs::create_dir(Path::new(".hamachi"))?;
    fs::create_dir(Path::new(".hamachi/objects"))?;
    fs::create_dir_all(Path::new(".hamachi/refs/heads"))?;
    fs::create_dir(Path::new(".hamachi/refs/tags"))?;
    
    Ok(())
}