use eyre::Result;
use serde_json::error::Category;
use std::ffi::OsStr;
use std::path::Path;
use std::{io, io::prelude::*};

mod format_parser;

#[derive(Debug)]
enum Issue {
    SyntaxError,
    EoFError,
}

fn parse_json(contents: &str) -> Result<Option<Issue>> {
    match serde_json::from_str::<serde_json::Value>(&contents) {
        Err(e) => match e.classify() {
            Category::Syntax => Ok(Some(Issue::SyntaxError)),
            Category::Eof => Ok(Some(Issue::EoFError)),
            Category::Io | Category::Data => Err(eyre::eyre!(e)),
        },
        Ok(_) => Ok(None),
    }
}

fn main() -> Result<()> {
    let json_extension = Some(OsStr::new("json"));

    let mut json_err = 0;
    let mut format_err = 0;

    println!("::group::errors_and_warnings");
    for line in io::stdin().lock().lines() {
        let l = line?;
        let path = Path::new(&l);
        // ignore none files and files with no '.josn' extension
        if !path.is_file() || path.extension() != json_extension {
            continue;
        }
        let contents = std::fs::read_to_string(path)?;
        if let Some(issue) = parse_json(&contents)? {
            println!("{:?} {:?}", &path, &issue);
            json_err += 1;
        } else if !format_parser::check_format(&contents) {
            println!("{:?} WrongFormat", &path);
            format_err += 1;
        }
    }
    println!("::endgroup::");

    if json_err + format_err == 0 {
        Ok(())
    } else {
        println!("Found");
        println!("     {json_err} JSON encoding errors and");
        println!("     {format_err} format errors.");
        std::process::exit(65)
    }
}
