mod object;
mod test_utils;
mod command;
mod remote;

use std::fs;
use std::fs::File;
use std::path::{Path, PathBuf};
use clap::Parser;
use rand::RngCore;
use command::{Args, Command};
use command::config::config;
use crate::command::cat_file::cat_file;
use crate::command::clone::clone;
use crate::command::commit_tree::commit_tree;
use crate::command::hash_object::hash_object;
use crate::command::ls_tree::ls_tree;
use crate::command::write_tree::write_tree;

fn main() {
    let args = Args::parse();

    if std::env::var("test") == Ok("true".to_string()) {
        let dir = Path::new("./testing").join(rand::rng().next_u64().to_string());
        fs::create_dir_all(&dir).unwrap();
        
        std::env::set_current_dir(&dir).unwrap();
        
        init().expect("Failed to init");
    }

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
            let commit_hash = commit_tree(&hash, &message).unwrap().to_string();
            
            println!("{commit_hash}");
        },
        Command::Config { subcommand } => {
            config(subcommand);
        },
        Command::Clone { repository } => {
            clone(repository);
        }
    }
}

/// Initialize a new git repository
/// https://git-scm.com/docs/git-init
fn init() -> std::io::Result<()> {
    fs::create_dir(Path::new(".hamachi"))?;
    fs::create_dir(Path::new(".hamachi/objects"))?;
    fs::create_dir(Path::new(".hamachi/objects/pack"))?;
    fs::create_dir_all(Path::new(".hamachi/refs/heads"))?;
    fs::create_dir(Path::new(".hamachi/refs/tags"))?;
    
    File::create(".hamachi/config")?;
    
    Ok(())
}