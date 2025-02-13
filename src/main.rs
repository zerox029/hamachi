use std::{fmt, fs};
use std::fmt::Formatter;
use std::io::Read;
use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        hash: String,
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            
        },
        Command::CatFile { pretty_print, hash } => {
            cat_file(pretty_print, &hash)
        }
    }
}

fn cat_file(pretty_print: bool, hash: &str) {
    let subdirectory = &hash[..2];
    let file_name = &hash[2..];
    
    let file_path = format!(".git/objects/{}/{}", subdirectory, file_name);
    let compressed_file_content = fs::read(&file_path).unwrap();
    let compressed_file_content = compressed_file_content.as_slice();
    
    let mut decompresser = ZlibDecoder::new(compressed_file_content);
    let mut decompressed_file_content = String::new();
    decompresser.read_to_string(&mut decompressed_file_content).unwrap();

    let components = decompressed_file_content.split('\0').collect::<Vec<&str>>();
    let header = components[0].split(" ").collect::<Vec<&str>>();

    if header[0] != "blob" {
        panic!("Not a blob")
    }

    let blob = Blob{size: header[1].parse::<usize>().unwrap(), data: components[1].to_string()};

    println!("{}", blob);
}

struct Blob {
    size: usize,
    data: String,
}

impl fmt::Display for Blob {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.data)
    }
}