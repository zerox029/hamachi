use crate::object::commit::Commit;
use crate::object::packfile::ObjectType;
use crate::remote::http_client::HttpClient;
use reqwest::Url;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;

/// Clone a repository into a new directory
/// https://git-scm.com/docs/git-clone
pub fn clone(repository: String) {
    let url = Url::parse(&repository).unwrap();
    let client = HttpClient::new(url);

    println!("Cloning into 'hamachi' ...");

    let discover_refs_response = client.discover_refs().unwrap();
    let packfile = client.fetch_pack(&discover_refs_response);

    println!("Packfile Headers");
    println!("- Signature {}", packfile.header.signature);
    println!("- Version {}", packfile.header.version);
    println!("- Entry Count {}", packfile.header.entry_count);

    // Setup HEAD
    let mut commits = HashSet::new();
    let mut parent_commits = HashSet::new();
    for entry in packfile.entries {
        if entry.object_type == ObjectType::Commit {
            let mut commit = Commit::from_hash(&entry.hash);

            commits.insert(entry.hash);
            for parent in commit.parents {
                parent_commits.insert(parent.parent_hash);
            }
        }
    }

    let commits = commits
        .symmetric_difference(&parent_commits)
        .collect::<Vec<_>>();
    let head_commit_hash = commits.get(0).unwrap();

    let mut master_ref_file = File::create(".hamachi/refs/heads/master").unwrap();
    master_ref_file
        .write_all(head_commit_hash.to_string().as_bytes())
        .unwrap();

    let mut head_file = File::create(".hamachi/HEAD").unwrap();
    head_file
        .write_all("ref: refs/heads/master".as_bytes())
        .unwrap();

    let head_commit = Commit::from_hash(head_commit_hash);
    println!("Head commit hash {}", head_commit_hash.to_string());
    println!("Head commit tree {}", head_commit.tree_hash.to_string());
}
