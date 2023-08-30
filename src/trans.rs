// SPDX-License-Identifier: MIT
use crate::{cmark_xml, deepl};

/// Translate CommonMark .md file
pub async fn translate_cmark_file<P: AsRef<std::path::Path>>(
    deepl: &deepl::Deepl,
    from_lang: deepl::Language,
    to_lang: deepl::Language,
    formality: deepl::Formality,
    src_path: P,
    dst_path: P,
) -> std::io::Result<()> {
    use std::io::Write;
    log::debug!("start translate. input: {}", &src_path.as_ref().display());

    // Read .md file
    let mut f = std::fs::File::open(&src_path)?;
    let (cmark_text, delimiter, frontmatter) = cmark_xml::read_cmark_with_frontmatter(&mut f)?;
    drop(f);

    log::debug!(
        "Read file:\n+++\ndelimiter: {}\n+++\nfrontmatter: {}\n+++\n{}",
        delimiter,
        frontmatter.as_deref().unwrap_or_default(),
        cmark_text
    );

    let is_md_file = src_path.as_ref().extension().is_some()
        && (src_path.as_ref().extension().unwrap() == "md"
            || src_path.as_ref().extension().unwrap() == "mdx");

    // If Deepl API KEY is a free version, get the number of characters remaining to be translated.
    if deepl.config.is_free_api_key() {
        api_availability_check(&deepl, &cmark_text).await?;
    }

    // Parse frontmatter. For Markdown files, do not translate front matter.
    let translated_frontmatter = match frontmatter {
        Some(frontmatter) if !is_md_file => {
            // translate TOML frontmatter
            Some(translate_toml(&deepl, from_lang, to_lang, formality, &frontmatter).await?)
        }
        Some(frontmatter) => Some(frontmatter),
        _ => None,
    };

    // Translate CommonMark body
    let translated_cmark =
        translate_cmark(&deepl, from_lang, to_lang, formality, &cmark_text).await?;

    // create output directory
    if let Some(parent) = dst_path.as_ref().parent() {
        std::fs::create_dir_all(parent)?;
    }

    // Print result
    // let mut f = std::fs::File::create(&dst_path)?;
    let mut write_string = String::new();
    if let Some(translated_frontmatter) = translated_frontmatter {
        write_string.push_str(format!("{}{}", delimiter, "\n").as_str());
        write_string.push_str(translated_frontmatter.as_str());
        write_string.push_str(format!("{}{}", delimiter, "\n").as_str());
    }
    write_string.push_str(translated_cmark.as_str());

    // deepl.config.backup_original_text が true の場合は原文をコメントアウトで残す。
    // 原文に"-->"が含まれていると原文全体のコメントが失敗するため"-!->"に置換する。
    if deepl.config.backup_original_text {
        write_string.push_str("\n<!---\n");
        write_string.push_str(&cmark_text.as_str().replace("-->", "-!->"));
        write_string.push_str("\n-->\n");
    }
    let mut f = std::fs::File::create(&dst_path)?;
    f.write_all(write_string.as_bytes())?;
    Ok(())
}

/// Translate TOML frontmatter
pub async fn translate_toml(
    deepl: &deepl::Deepl,
    from_lang: deepl::Language,
    to_lang: deepl::Language,
    formality: deepl::Formality,
    toml_frontmatter: &str,
) -> Result<String, std::io::Error> {
    if let toml::Value::Table(mut root) = toml_frontmatter.parse::<toml::Value>()? {
        // Pickup TOML key for translation
        let mut should_be_translate: Vec<&mut String> = vec![];
        for (key, val) in &mut root {
            match key.as_str() {
                "title" | "description" => {
                    if let toml::Value::String(val) = val {
                        should_be_translate.push(val);
                    }
                }
                "extra" => {
                    if let toml::Value::Table(extra) = val {
                        for (extra_key, extra_val) in extra {
                            match extra_key.as_str() {
                                "time" => {
                                    if let toml::Value::String(extra_val) = extra_val {
                                        should_be_translate.push(extra_val);
                                    }
                                }
                                _ => (),
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Prepare input Vec
        let src_vec = should_be_translate
            .iter()
            .map(|s| s.as_str())
            .collect::<Vec<&str>>();

        // If Deepl API KEY is a free version, get the number of characters remaining to be translated.
        if deepl.config.is_free_api_key() {
            api_availability_check(&deepl, &src_vec.join("")).await?;
        }

        // Translate texts
        let translated_vec = deepl
            .translate_strings(from_lang, to_lang, formality, &src_vec)
            .await
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;

        // Replace TOML value with translated text
        should_be_translate
            .into_iter()
            .zip(translated_vec.iter())
            .for_each(|(toml_val, translated_str)| {
                toml_val.clear();
                *toml_val += translated_str.as_str();
            });

        // Serialize toml::Value should not fail
        let translated_frontmatter = toml::to_string_pretty(&toml::Value::Table(root)).unwrap();
        // Show translated frontmatter
        log::trace!("Translated TOML :\n{}\n", translated_frontmatter);

        Ok(translated_frontmatter)
    } else {
        // TOML parse failed
        Err(std::io::Error::from(std::io::ErrorKind::InvalidData))
    }
}

/// Translate CommonMark
pub async fn translate_cmark(
    deepl: &deepl::Deepl,
    from_lang: deepl::Language,
    to_lang: deepl::Language,
    formality: deepl::Formality,
    cmark_text: &str,
) -> Result<String, std::io::Error> {
    let xml = cmark_xml::xml_from_cmark(&cmark_text, true);
    log::trace!("XML: {}\n", xml);

    let target_name = deepl.config.project_name.as_str();
    let xml = deepl::Deepl::add_ignore_tags(deepl, target_name, &xml).await;

    log::trace!("111111 added ignore tags. XML: {}\n", xml);

    // If Deepl API KEY is a free version, get the number of characters remaining to be translated.
    if deepl.config.is_free_api_key() {
        api_availability_check(&deepl, &xml).await?;
    }

    // translate
    let xml_translated = deepl
        .translate_xml(from_lang, to_lang, formality, target_name, &xml)
        .await
        .unwrap();

    // write back to markdown format
    log::trace!(
        "222222 Translated XML(before remove ignore tags): {}\n",
        &xml_translated
    );

    let xml_translated = deepl::Deepl::remove_ignore_tags(deepl, &xml_translated).await;
    log::trace!(
        "333333 Translated XML(after remove ignore tags): {}\n",
        &xml_translated
    );

    let cmark_translated = cmark_xml::cmark_from_xml(&xml_translated, true).unwrap();

    log::trace!("444444 cmark_translated: {}\n", &cmark_translated);

    Ok(cmark_translated)
}

async fn api_availability_check(deepl: &deepl::Deepl, text: &str) -> Result<bool, std::io::Error> {
    let used_chars = deepl.get_usage().await.unwrap() as usize;
    let remaining_chars = deepl::MAX_TRANSLATE_LENGTH - used_chars;
    log::info!("Remaining characters: {}", remaining_chars);
    if remaining_chars < text.len() {
        let error_message = format!(
            "The number of characters to be translated exceeds the limit. {}/{} Used.",
            used_chars,
            deepl::MAX_TRANSLATE_LENGTH
        );
        log::error!("{}", error_message);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            error_message,
        ));
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[tokio::test]
    async fn test_translate_cmark_file() -> Result<(), Box<dyn std::error::Error>> {
        // Load Deepl configuration from "deepl.toml"
        let deepl = deepl::Deepl::with_config("deepl.toml").unwrap();

        let from_lang = deepl::Language::En;
        let to_lang = deepl::Language::Ja;
        let formality = deepl::Formality::Formal;

        // Prepare temporary directory for testing
        let tests_dir = PathBuf::from("./tests");
        let src_path = tests_dir.as_path().join("test.md");
        let dst_path = tests_dir.as_path().join("translated.md");
        std::fs::write(
            &src_path,
            "+++\ntitle = \"Hello World\"\n+++\nThis is a test.",
        )?;

        // Call the function to be tested
        // APIの使用上限に達するとエラーになる。
        translate_cmark_file(&deepl, from_lang, to_lang, formality, &src_path, &dst_path)
            .await
            .unwrap();

        // Check if the translated content is as expected
        let translated_content = std::fs::read_to_string(&dst_path)?;
        let expected_content = "+++\ntitle = \"こんにちは世界\"\n+++\nこれはテストです。\n<!---\nThis is a test.\n-->\n";
        assert_eq!(translated_content, expected_content);

        Ok(())
    }

    #[tokio::test]
    async fn test_translate_toml() -> Result<(), Box<dyn std::error::Error>> {
        // Load Deepl configuration from "deepl.toml"
        let deepl = deepl::Deepl::with_config("deepl.toml").unwrap();

        let from_lang = deepl::Language::En;
        let to_lang = deepl::Language::Ja;
        let formality = deepl::Formality::Formal;

        let toml_frontmatter = r#"title = "Hello World"
                                        description = "Description"
                                        [extra]
                                        time = "2023-03-10""#;

        let translated_frontmatter =
            translate_toml(&deepl, from_lang, to_lang, formality, toml_frontmatter).await?;
        let expected_translated = r#"title = "こんにちは世界"
                                            description = "説明"
                                            [extra]
                                            time = "2023年3月10日""#;
        assert_eq!(translated_frontmatter, expected_translated);

        Ok(())
    }

    #[tokio::test]
    async fn test_translate_cmark() -> Result<(), Box<dyn std::error::Error>> {
        // Load Deepl configuration from "deepl.toml"
        let deepl = deepl::Deepl::with_config("deepl.toml").unwrap();

        let from_lang = deepl::Language::En;
        let to_lang = deepl::Language::Ja;
        let formality = deepl::Formality::Formal;

        let cmark_text = "This is a test.";
        let translated_cmark =
            translate_cmark(&deepl, from_lang, to_lang, formality, cmark_text).await?;
        let expected_translated = "これはテストです。";
        assert_eq!(translated_cmark, expected_translated);

        Ok(())
    }
}
