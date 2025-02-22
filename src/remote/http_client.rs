use crate::object::packfile::PackFile;
use reqwest::Url;
use std::io::Read;

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

    pub fn fetch_pack(&self, discover_refs_response: &DiscoverRefsResponse) -> PackFile {
        let pack = generate_pack(discover_refs_response);

        let mut upload_response = reqwest::blocking::Client::new()
            .post(format!("{}/git-upload-pack", self.url))
            .header("Content-Type", "application/x-git-upload-pack-request")
            .body(pack)
            .send()
            .unwrap();

        assert!(upload_response.status().is_success());

        let mut data: Vec<u8> = Vec::new();
        upload_response.read_to_end(&mut data).unwrap();

        PackFile::new(
            data,
            discover_refs_response.want.get(0).unwrap().hash.to_owned(),
        )
    }
}

fn parse_discover_refs_response(string: String) -> DiscoverRefsResponse {
    let advertised = string
        .strip_prefix("001e# service=git-upload-pack\n0000")
        .unwrap()
        .strip_suffix("\n0000")
        .unwrap()
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
    let want = vec![initial_connection_response.want.get(0).unwrap()];
    let want_section = want
        .iter()
        .map(|w| {
            let pkt_line = format!("want {}\n", w.hash);
            format!("{:0>4x}{}", pkt_line.len() + 4, pkt_line)
        })
        .collect::<String>();

    let have_section = initial_connection_response
        .common
        .iter()
        .map(|h| format!("0032have {}\n", h.hash))
        .collect::<String>();
    let pack = format!("{want_section}{have_section}00000009done\n");

    pack
}

/// Parse the variable length integers used to encode the length of packfile objects.
/// The MSB of each byte n of the number is 1 if the byte n+1 is also part of the number

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
