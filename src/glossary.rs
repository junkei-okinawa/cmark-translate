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

#[cfg(test)]
mod tests {
    use super::*;
    use std::{path::PathBuf, thread::sleep, time::Duration};

    // Helper function to create a temporary file with the given content
    fn create_temp_file(content: &str) -> (PathBuf, PathBuf) {
        let tests_dir = PathBuf::from("./tests");
        let test_file_path = tests_dir.as_path().join("test.toml");
        std::fs::write(&test_file_path, content).unwrap();
        (tests_dir, test_file_path)
    }

    #[test]
    fn test_read_file() {
        // Create a temporary file with content
        let content = "Hello, this is a test content.";
        let (_, test_file_path) = create_temp_file(content);

        // Call the function to be tested
        let result = read_file(&test_file_path);

        // Check if the result matches the expected content
        assert_eq!(result, Ok(content.to_string()));
    }

    #[test]
    fn test_read_glossary() {
        // To use the same file for multiple tests. Delay processing to avoid errors caused by replacing content in files.
        sleep(Duration::from_secs(1));

        // Prepare test TOML content with glossary
        let toml_content = r#"
[glossaries]
colors = { "red" = "赤", "blue" = "青" }
numbers = { "one" = "一", "two" = "二" }
"#;
        let (_tests_dir, test_file_path) = create_temp_file(toml_content);

        // Call the function to be tested
        let result = read_glossary("colors", &test_file_path);

        // Check if the result matches the expected glossary entries
        let expected_glossary = vec![
            ("blue".to_string(), "青".to_string()),
            ("red".to_string(), "赤".to_string()),
        ];
        assert_eq!(result.unwrap(), expected_glossary);
    }
}
