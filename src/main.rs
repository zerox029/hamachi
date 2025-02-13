use std::{fmt, fs};
use std::fmt::{format, Formatter};
use std::fs::File;
use std::io::{Read, Write};
use clap::{Parser, Subcommand};
use flate2::Compression;
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use sha1::{Digest, Sha1};

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
    },
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        file: String,
    }
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init => {

        },
        Command::CatFile { pretty_print, hash } => {
            cat_file(pretty_print, &hash);
        },
        Command::HashObject { write, file } => {
            hash_object(write, &file);
        }
    }
}

fn cat_file(_pretty_print: bool, hash: &str) {
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

fn hash_object(write: bool, file: &str) {
    let uncompressed_file_content = fs::read(&file).unwrap();

    let header = format!("blob {}", uncompressed_file_content.len());
    let blob = format!("{}\0{}", header, String::from_utf8(uncompressed_file_content).unwrap());

    let mut hasher = Sha1::new();
    Digest::update(&mut hasher, &blob);
    let hash = hex::encode(hasher.finalize());

    println!("{}", hash);

    if write {
        let mut compresser = ZlibEncoder::new(Vec::new(), Compression::default());
        compresser.write_all(&blob.as_bytes()).unwrap();
        let compressed_bytes = compresser.finish().unwrap();

        let subdirectory = &hash[..2];
        let file_name = &hash[2..];
        let file_path = format!(".git/objects/{}/{}", subdirectory, file_name);

        println!("file_path: {}", file_path);

        fs::create_dir_all(format!(".git/objects/{}", subdirectory)).unwrap();
        let mut file = File::create(file_path).unwrap();
        file.write_all(&compressed_bytes).unwrap();
    }
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