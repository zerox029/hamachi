mod cli;
mod object;

use std::path::PathBuf;
use clap::Parser;
use cli::{Args, Command};
use object::blob::{cat_file, hash_object};
use object::tree::ls_tree;
use crate::object::tree::write_tree;

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init => {

        },
        Command::CatFile { pretty_print, hash } => {
            let _ = cat_file(pretty_print, &hash);
        },
        Command::HashObject { write, file } => {
            let _ = hash_object(write, &PathBuf::from(file));
        },
        Command::LsTree { name_only, hash } => {
            let _ = ls_tree(name_only, &hash);
        },
        Command::WriteTree => {
            let _ = write_tree(None);
        }
    }
}