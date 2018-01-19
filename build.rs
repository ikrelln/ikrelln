use std::fs::File;
use std::io::prelude::*;
use std::process::Command;

fn main() {
    let output = Command::new("git")
        .arg("rev-parse HEAD")
        .output()
        .expect("failed to execute process");
    let gitref = match String::from_utf8(output.stdout)
        .unwrap_or("N/A".to_string())
        .as_ref()
    {
        "" => "N/A".to_string(),
        v => v.to_string(),
    };
    let version = env!("CARGO_PKG_VERSION");

    let new_content = format!(
        "
pub struct BuildInfo {{
    pub version: &'static str,
    pub git: &'static str,
}}

lazy_static! {{
    pub static ref BUILD_INFO: BuildInfo = BuildInfo {{
        version: \"{}\",
        git: \"{}\",
    }};
}}
",
        version, gitref
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
