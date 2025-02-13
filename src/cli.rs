use clap::{Parser, Subcommand};

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
    }
}