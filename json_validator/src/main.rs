use clap::{Parser, Subcommand};
use eyre::Result;
use serde_json::error::Category;
use std::ffi::OsStr;
use std::path::Path;
use std::{io, io::prelude::*};

mod format_parser;

#[derive(Debug)]
enum Issue {
    SyntaxError,
}

fn parse_json(contents: &str) -> Result<Option<Issue>> {
    match serde_json::from_str::<serde_json::Value>(&contents) {
        Err(e) => match e.classify() {
            Category::Syntax => Ok(Some(Issue::SyntaxError)),
            Category::Io | Category::Data | Category::Eof => Err(eyre::eyre!(e)),
        },
        Ok(_) => Ok(None),
    }
}

fn main() -> Result<()> {
    //let args = Args::parse();
    let dot_json = Some(OsStr::new("json"));

    let mut issues = 0;

    for line in io::stdin().lock().lines() {
        let l = line?;
        let path = Path::new(&l);
        // ignore none files and files with no '.josn' extension
        if !path.is_file() || path.extension() != dot_json {
            continue;
        }
        let contents = std::fs::read_to_string(path)?;
        if let Some(issue) = parse_json(&contents)? {
            println!("{:?} {:?}", &path, &issue);
            issues += 1;
        } else if !format_parser::check_format(&contents) {
            println!("{:?} WrongFormat", &path);
            issues += 1;
        }
    }

    if issues == 0 {
        Ok(())
    } else {
        std::process::exit(65)
    }
}
