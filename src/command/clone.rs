use reqwest::Url;
use crate::remote::http_client::{HttpClient};

/// Clone a repository into a new directory
/// https://git-scm.com/docs/git-clone
pub fn clone(repository: String) {
    let url = Url::parse(&repository).unwrap();
    let client = HttpClient::new(url);
    
    println!("Cloning into 'hamachi' ...");

    let discover_refs_response = client.discover_refs().unwrap();
    client.fetch_pack(&discover_refs_response);
}