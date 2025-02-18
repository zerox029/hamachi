use std::io::{BufReader, Read};
use flate2::read::ZlibDecoder;
use reqwest::blocking::Response;
use reqwest::Url;
use crate::object::Object;

pub struct HttpClient {
    url: Url,
    reqwest_client: reqwest::blocking::Client,
}

impl HttpClient {
    pub fn new(url: Url) -> HttpClient {
        HttpClient {
            url,
            reqwest_client: reqwest::blocking::Client::new(),
        }
    }

    pub fn discover_refs(&self) -> Result<DiscoverRefsResponse, &'static str> {
        let url = format!("{}/info/refs?service=git-upload-pack", self.url);
        let response = self.reqwest_client.get(url).send().unwrap();

        if response.status().is_success() {
            let discover_refs = parse_discover_refs_response(response.text().unwrap());

            return Ok(discover_refs);
        }

        Err("Failed to discover refs")
    }

    pub fn fetch_pack(&self, discover_refs_response: &DiscoverRefsResponse) {
        let pack = generate_pack(discover_refs_response);

        let mut upload_response = reqwest::blocking::Client::new()
            .post(format!("{}/git-upload-pack", self.url))
            .header("Content-Type", "application/x-git-upload-pack-request")
            .body(pack)
            .send()
            .unwrap();

        assert!(upload_response.status().is_success());

        let mut data: Vec<u8> = Vec::new();

        data.resize(8, 0);
        upload_response.read_exact(&mut data).unwrap();

        // 4 byte signature PACK
        data.clear();
        data.resize(4, 0);
        upload_response.read_exact(&mut data).unwrap();

        // Version number
        data.clear();
        data.resize(4, 0);
        upload_response.read_exact(&mut data).unwrap();

        let version = u32::from_be_bytes(data.clone().try_into().unwrap());

        // Number of objects
        data.clear();
        data.resize(4, 0);
        upload_response.read_exact(&mut data).unwrap();

        let item_count = u32::from_be_bytes(data.clone().try_into().unwrap());

        // Objects
        parse_object_header(&mut upload_response);
        
        data.clear();
        upload_response.read_to_end(&mut data).unwrap();
        
        let mut decompressor = ZlibDecoder::new(&*data);
        let mut decompressed_data = String::new();
        decompressor.read_to_string(&mut decompressed_data).unwrap();
        
        println!("Received object {} of {}...", 1, item_count);
        println!("{}", decompressed_data);
    }
}

fn parse_discover_refs_response(string: String) -> DiscoverRefsResponse {
    let advertised = string
        .strip_prefix("001e# service=git-upload-pack\n0000").unwrap()
        .strip_suffix("\n0000").unwrap()
        .split('\n')
        .map(|s| {
            let (hash, rest) = s.split_once(" ").unwrap();
            let (name, params) = rest.split_once("\0").unwrap_or((rest, ""));

            let hash = hash[4..].to_owned();
            let name = name.to_owned();
            let params = params.to_owned();

            Ref { hash, name, params }
        })
        .collect::<Vec<_>>();

    let common: Vec<Ref> = Vec::new(); // Empty for now
    let want = advertised; // TODO: Needs to be the difference of avertised and common

    DiscoverRefsResponse { common, want }
}

fn generate_pack(initial_connection_response: &DiscoverRefsResponse) -> String {
    let want_section = initial_connection_response.want
        .iter()
        .map(|w| {
            let pkt_line = format!("want {}\n", w.hash);
            format!("{:0>4x}{}", pkt_line.len() + 4, pkt_line)})
        .collect::<String>();

    let have_section = initial_connection_response.common.iter().map(|h| format!("0032have {}\n", h.hash)).collect::<String>();
    let pack = format!("{want_section}{have_section}00000009done\n");

    pack
}

/// Parse the variable length integers used to encode the length of packfile objects.
/// The MSB of each byte n of the number is 1 if the byte n+1 is also part of the number
fn parse_object_header(response: &mut Response) -> (ObjectType, Vec<u8>) {
    let mut data = Vec::new();
    data.resize(1, 0);
    response.read_exact(&mut data).unwrap();

    let mut is_final_byte = *data.get(0).unwrap() < 128u8;
    let object_type = ObjectType::from_u8(*data.get(0).unwrap() >> 4 & 0b111).unwrap();
    let mut size_bits = Vec::new();
    size_bits.push(*data.get(0).unwrap() & 0b1111);

    while !is_final_byte {
        data.clear();
        data.resize(1, 0);
        response.read_exact(&mut data).unwrap();

        is_final_byte = *data.get(0).unwrap() < 128u8;
        size_bits.push(*data.get(0).unwrap() & 0b1111111);
    }

    (object_type, size_bits)
}

#[derive(Debug)]
pub struct Ref {
    pub(crate) hash: String,
    name: String,
    params: String,
}

pub struct DiscoverRefsResponse {
    pub(crate) common: Vec<Ref>,
    pub(crate) want: Vec<Ref>,
}

#[derive(Debug)]
enum ObjectType {
    Commit = 1,
    Tree = 2,
    Blob = 3,
    Tag = 4,
    OfsDelta = 6,
    RefDelta = 7,
}
impl ObjectType {
    fn from_u8(value: u8) -> Result<ObjectType, &'static str> {
        match value {
            1 => Ok(ObjectType::Commit),
            2 => Ok(ObjectType::Tree),
            3 => Ok(ObjectType::Blob),
            4 => Ok(ObjectType::Tag),
            6 => Ok(ObjectType::OfsDelta),
            7 => Ok(ObjectType::RefDelta),
            _ => Err("Unknown object type"),
        }
    }
}
