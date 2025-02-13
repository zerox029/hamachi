mod cli;
mod blob;

use clap::Parser;
use cli::{Args, Command};
use blob::{cat_file, hash_object};

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
        }
    }
}