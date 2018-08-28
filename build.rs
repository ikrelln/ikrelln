use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

struct Ignore;

impl<E> From<E> for Ignore
where
    E: Error,
{
    fn from(_: E) -> Ignore {
        Ignore
    }
}

fn commit_hash() -> Result<String, Ignore> {
    Ok(try!(String::from_utf8(
        try!(Command::new("git").args(&["rev-parse", "HEAD"]).output()).stdout
    )))
}

fn commit_describe() -> Result<String, Ignore> {
    Ok(try!(String::from_utf8(
        try!(
            Command::new("git")
                .args(&["describe", "--all", "--dirty", "--long"])
                .output()
        ).stdout
    )))
}

fn commit_date() -> Result<String, Ignore> {
    Ok(try!(String::from_utf8(
        try!(
            Command::new("git")
                .args(&["log", "-1", "--pretty=format:%cI"])
                .output()
        ).stdout
    )))
}

fn main() {
    let gitref = match commit_hash() {
        Ok(v) => v.trim_right().to_string(),
        Err(_) => "N/A".to_string(),
    };
    let gitdate = match commit_date() {
        Ok(v) => v.trim_right().to_string(),
        Err(_) => "N/A".to_string(),
    };
    let gitdescribe = match commit_describe() {
        Ok(v) => v.trim_right().to_string(),
        Err(_) => "N/A".to_string(),
    };
    let version = env!("CARGO_PKG_VERSION");

    let new_content = format!(
        "#[derive(Serialize, Debug, Clone)]
pub struct BuildInfo {{
    pub version: &'static str,
    pub commit_hash: &'static str,
    pub commit_date: &'static str,
    pub commit_describe: &'static str,
}}

pub static BUILD_INFO: BuildInfo = BuildInfo {{
    version: \"{}\",
    commit_hash: \"{}\",
    commit_date: \"{}\",
    commit_describe: \"{}\",
}};
",
        version, gitref, gitdate, gitdescribe
    );

    let update = File::open("src/build_info.rs")
        .map(|mut f| {
            let mut contents = String::new();
            f.read_to_string(&mut contents).unwrap();
            return contents;
        })
        .map(|content| content != new_content)
        .unwrap_or(true);

    if update {
        let mut file = File::create("src/build_info.rs").unwrap();
        file.write_all(new_content.as_bytes()).unwrap();
    }
}
