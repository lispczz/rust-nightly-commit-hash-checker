use anyhow::anyhow;
use serde_json::json;

const NIGHTLY_MANIFEST_URL: &str = "https://static.rust-lang.org/dist/channel-rust-nightly.toml";
const GRAPHQL_ENDPOINT: &str = "https://api.github.com/graphql";
const GRAPHQL_TEMPLATE: &str = r#"
{
  repository(owner: "rust-lang", name: "rust") {
    object(expression: "{{commit}}") {
      ... on Commit {
        history(first:100, {{#if cursor}}after: "{{cursor}}"{{/if}}) {
          nodes {
            id
            oid
            committedDate
          }
          pageInfo {
            endCursor
          }
        }
      }
    }
  }
}
"#;

fn get_nightly_commit_hash() -> anyhow::Result<(String, String)> {
    let body = reqwest::blocking::get(NIGHTLY_MANIFEST_URL)?.text()?;
    let parsed_toml = body.parse::<toml::Value>()?;
    let manifest = parsed_toml
        .as_table()
        .ok_or_else(|| anyhow!("cannot parse manifest"))?;
    let version = manifest["pkg"]["rust"]["version"]
        .as_str()
        .ok_or_else(|| anyhow!("cannot find version"))?
        .to_string();
    let hash = manifest["pkg"]["rust"]["git_commit_hash"]
        .as_str()
        .ok_or_else(|| anyhow!("cannot find hash"))?
        .to_string();
    Ok((version, hash))
}

fn check_commit(root: &str, target: &str) -> anyhow::Result<bool> {
    let template = handlebars::Handlebars::new();
    let client = reqwest::blocking::Client::new();
    let mut cursor = "".to_string();
    let mut trial_count = 0;
    while trial_count < 5 {
        let req = template.render_template(
            GRAPHQL_TEMPLATE,
            &json!({
                "cursor": cursor,
                "commit": root,
            }),
        )?;
        let res_raw: String = client
            .post(GRAPHQL_ENDPOINT)
            .header("User-Agent", dotenv::var("USER_NAME")?)
            .bearer_auth(dotenv::var("USER_TOKEN")?)
            .json(&json!({ "query": req }))
            .send()?
            .text()?;
        let res_json: serde_json::Value =
            serde_json::from_str(res_raw.as_str()).map_err(|err| {
                eprintln!("invalid response {}", res_raw);
                err
            })?;
        let history = &res_json["data"]["repository"]["object"]["history"];
        cursor = history["pageInfo"]["endCursor"]
            .as_str()
            .ok_or_else(|| anyhow!("invalid response format"))?
            .to_string();
        let commits: Vec<String> = history["nodes"]
            .as_array()
            .ok_or_else(|| anyhow!("invalid response format"))?
            .iter()
            .map(|value| value["oid"].as_str().unwrap_or("").to_string())
            .collect();
        if commits.iter().any(|value| value.eq(target)) {
            return Ok(true);
        }
        trial_count += 1;
    }
    Ok(false)
}

fn main() {
    dotenv::dotenv().ok();

    let nightly = get_nightly_commit_hash();
    if let Err(err) = nightly {
        eprintln!("cannot get nightly version. Err: {}", err);
        return;
    }
    let (version, hash) = nightly.unwrap();
    println!("nightly:");
    println!("version: {}", version);
    println!("hash: {}", hash);

    if let Some(commit) = std::env::args().nth(1) {
        match check_commit(&hash, &commit) {
            Err(err) => {
                eprintln!("check commit failed: {}", err);
                return;
            }
            Ok(is_nightly_contain_this_commit) => {
                if is_nightly_contain_this_commit {
                    println!("Found! Nightly build contains this commit: {}", &commit);
                } else {
                    println!(
                        "Not found! Nightly build doesn't contain this commit: {}",
                        &commit
                    );
                }
            }
        }
    }
}
