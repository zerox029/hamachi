use std::io::{Read};

/// Clone a repository into a new directory
/// https://git-scm.com/docs/git-clone
pub fn clone(repository: String) {
    let repository = repository.strip_suffix('/').unwrap_or(&repository);

    let response = reqwest::blocking::get(format!("{}/info/refs?service=git-upload-pack",repository)).unwrap();

    if response.status() == reqwest::StatusCode::OK {
        let initial_connection_response = parse_initial_response(response.text().unwrap());
        let pack = generate_pack(&initial_connection_response);

        println!("{pack:?}");

        let mut upload_response = reqwest::blocking::Client::new()
            .post(format!("{}/git-upload-pack", repository))
            .header("Content-Type", "application/x-git-upload-pack-request")
            .body(pack)
            .send()
            .unwrap();
        
        assert!(upload_response.status().is_success());
        
        let mut data: Vec<u8> = Vec::new();
        upload_response.read_to_end(&mut data).unwrap();
        
        println!("{:?}", data);

        // let mut start = usize::from_str_radix(std::str::from_utf8(&data[..4]).unwrap(), 16).unwrap();
        // 
        // println!("{}", std::str::from_utf8(&data[..1]).unwrap());
        // 
        // assert_eq!(
        //     data[start..start + 4],
        //     ['P' as u8, 'A' as u8, 'C' as u8, 'K' as u8]
        // );
    }
    else {
        println!("{}", response.status());
    }
}

fn parse_initial_response(string: String) -> InitialConnectionResponse {
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

    InitialConnectionResponse { common, want }
}

fn generate_pack(initial_connection_response: &InitialConnectionResponse) -> String {
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

#[derive(Debug)]
struct Ref {
    hash: String,
    name: String,
    params: String,
}

struct InitialConnectionResponse {
    common: Vec<Ref>,
    want: Vec<Ref>,
}