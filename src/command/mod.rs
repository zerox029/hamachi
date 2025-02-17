use clap::{Parser, Subcommand};
use config::ConfigSubcommand;

pub mod config;
pub mod cat_file;
pub mod hash_object;
pub mod ls_tree;
pub mod write_tree;
pub mod commit_tree;
pub mod clone;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub(crate) struct Args {
    #[command(subcommand)]
    pub(crate) command: Command,
}

#[derive(Subcommand, Debug)]
pub(crate) enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        hash: String,
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: String,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,

        hash: String,
    },
    WriteTree,
    CommitTree {
        hash: String,

        #[clap(short = 'm')]
        message: Option<String>,
    },
    Config {
        #[clap(subcommand)]
        subcommand: ConfigSubcommand,
    },
    Clone {
        repository: String,
    },
}
