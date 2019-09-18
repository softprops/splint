use lazy_static::lazy_static;
use reqwest::Client;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::path::Path;
use std::process::exit;
use structopt::StructOpt;

const CATALOG: &str = include_str!("../data/catalog.json");

lazy_static! {
    static ref SCHEMA_STORE: Vec<Schema> = serde_json::from_str::<SchemaStore>(CATALOG)
        .expect("unparsable schema store")
        .schemas;
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct Schema {
    name: String,
    description: String,
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

fn main() {
    if let Err(err) = run() {
        eprintln!("{}", err);
        exit(1)
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let Opts { schema, files } = Opts::from_args();
    let provided = match schema {
        Some(location) => match &location[..] {
            url if url.starts_with("http") => Some(remote(url)?),
            path => Some(local(path)?),
        },
        _ => None,
    };
    for file in files {
        if let Some(prov) = &provided {
            for err in jsonschema_valid::validate(&local(file)?, &prov, None, true).get_errors() {
                println!("{}", err);
            }
        } else {
            for schema in SCHEMA_STORE.clone() {
                for file_matcher in schema.file_match {
                    if glob::Pattern::new(&file_matcher)?.matches(&file) {
                        for err in jsonschema_valid::validate(
                            &local(&file)?,
                            &remote(&schema.url)?,
                            None,
                            true,
                        )
                        .get_errors()
                        {
                            println!("{}", err);
                        }
                    }
                }
            }
        }
    }

    Ok(())
}
