// SPDX-License-Identifier: MIT
//!
//! Read glossaries from .xlsx
//!

use std::fs;
use std::io::{BufReader, Read};
use toml;

fn read_file(path: &std::path::Path) -> Result<String, String> {
    let mut file_content = String::new();

    let mut fr = fs::File::open(path)
        .map(|f| BufReader::new(f))
        .map_err(|e| e.to_string())?;

    fr.read_to_string(&mut file_content)
        .map_err(|e| e.to_string())?;

    Ok(file_content)
}

pub fn read_glossary<P: AsRef<std::path::Path>>(
    name: &str,
    path: P,
) -> Result<Vec<(String, String)>, toml::de::Error> {
    let s = match read_file(path.as_ref()) {
        Ok(s) => s,
        Err(e) => panic!("fail to read file: {}", e),
    };
    let toml_reslut = toml::from_str(&s);
    match toml_reslut {
        Ok(v) => {
            let mut glossary = Vec::new();
            let toml_value: toml::Value = v;
            let toml_map = toml_value.as_table().unwrap();
            if toml_map.get("glossaries").is_none() {
                panic!("fail to parse toml...");
            }
            let glossaries_value = toml_map.get("glossaries").unwrap();
            println!("{:?}", glossaries_value);
            if glossaries_value.get(name).is_none() {
                panic!("fail to get glossary name...");
            }
            let glossary_value = glossaries_value.get(name).unwrap();
            let glossary_map = glossary_value.as_table().unwrap();
            for g in glossary_map {
                let (from, to) = g;
                println!("{} -> {}", from, &to.to_string().replace("\"", ""));
                glossary.push((from.to_string(), to.to_string().replace("\"", "")));
            }
            Ok(glossary)
        }
        Err(e) => panic!("fail to parse toml: {}", e),
    }
}
