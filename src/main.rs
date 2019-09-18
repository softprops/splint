use glob::Pattern;
use jsonschema_valid::{validate, ValidationError};
use lazy_static::lazy_static;
use regex::Regex;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::{error::Error, fs::File, path::Path, process::exit};
use structopt::StructOpt;

lazy_static! {
    static ref SCHEMA_STORE: Vec<Schema> =
        serde_json::from_str::<SchemaStore>(include_str!("../data/catalog.json"))
            .expect("unparsable schema store")
            .schemas;
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Schema {
    name: String,
    description: String,
    #[serde(default)]
    file_match: Vec<String>,
    url: String,
}

#[derive(Deserialize)]
struct SchemaStore {
    schemas: Vec<Schema>,
}

#[derive(StructOpt)]
#[structopt(
    name = "splint",
    about = "ensures structures with a well defined shape stay in place"
)]
struct Opts {
    #[structopt(short = "s", long = "schema", help = "json schema to apply")]
    schema: Option<String>,
    #[structopt(help = "list of files to lint")]
    files: Vec<String>,
}

fn remote<U>(url: U) -> Result<Value, Box<dyn Error>>
where
    U: AsRef<str>,
{
    Ok(Client::new().get(url.as_ref()).send()?.json()?)
}

fn local<P>(path: P) -> Result<Value, Box<dyn Error>>
where
    P: AsRef<Path>,
{
    Ok(serde_yaml::from_reader(File::open(path)?)?)
}

fn schema(opts: &Opts) -> Result<Option<Value>, Box<dyn Error>> {
    let Opts { schema, .. } = opts;
    Ok(match schema {
        Some(location) => match &location[..] {
            url if url.starts_with("http") => Some(remote(url)?),
            path => Some(local(path)?),
        },
        _ => None,
    })
}

fn main() {
    if let Err(err) = lint(Opts::from_args()) {
        eprintln!("{}", err);
        exit(1)
    }
}

fn fmt(err: &ValidationError) -> String {
    // work around until https://github.com/mdboom/jsonschema-valid/issues/2
    lazy_static! {
        static ref RE: Regex =
            Regex::new(r#"At (.+) with schema at (.+): (.+)"#).expect("invalid regex");
    };
    let err_str = err.to_string();
    let caps = RE
        .captures(err_str.as_str())
        .unwrap_or_else(|| panic!("{} didn't match format", err_str));
    let field = caps.get(1).map(|c| c.as_str()).unwrap_or_default();
    let msg = caps.get(3).map(|c| c.as_str()).unwrap_or_default();
    format!("{}: {}", field, msg)
}

fn lint(opts: Opts) -> Result<(), Box<dyn Error>> {
    let provided = schema(&opts)?;
    let Opts { files, .. } = opts;
    let errors: Result<usize, Box<dyn Error>> =
        files.into_iter().try_fold(0, |mut errors, file| {
            if let Some(prov) = &provided {
                for err in validate(&local(&file)?, &prov, None, true).get_errors() {
                    eprintln!("{} {}", file, fmt(err));
                    errors += 1;
                }
            } else {
                for schema in SCHEMA_STORE.iter() {
                    for file_matcher in schema.file_match.iter() {
                        if Pattern::new(&file_matcher)?.matches(&file) {
                            for err in validate(&local(&file)?, &remote(&schema.url)?, None, true)
                                .get_errors()
                            {
                                eprintln!("{} {}", file, fmt(err));
                                errors += 1;
                            }
                        }
                    }
                }
            }
            Ok(errors)
        });

    if errors? > 0 {
        exit(1);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_file_matches_compile() {
        for schema in SCHEMA_STORE.clone() {
            for file_match in schema.file_match {
                assert!(Pattern::new(&file_match).is_ok())
            }
        }
    }

    #[test]
    fn test_file_matches() {
        for (file, expect) in &[(".angular-cli.json", ".angular-cli.json")] {
            assert_eq!(
                SCHEMA_STORE.iter().find_map(|value| value
                    .file_match
                    .iter()
                    .find(|pat| Pattern::new(pat).unwrap().matches(file))
                    .map(|_| value.name.clone())),
                Some(expect.to_string())
            )
        }
    }
}
