mod cli;
mod object;

use clap::Parser;
use cli::{Args, Command};
use object::blob::{cat_file, hash_object};
use object::tree::ls_tree;

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init => {

        },
        Command::CatFile { pretty_print, hash } => {
            let _ = cat_file(pretty_print, &hash);
        },
        Command::HashObject { write, file } => {
            let _ = hash_object(write, &file);
        },
        Command::LsTree { name_only, hash } => {
            let _ = ls_tree(name_only, &hash);
        }
    }
}