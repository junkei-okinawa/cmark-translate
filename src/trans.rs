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
    let mut f = std::fs::File::create(&dst_path)?;
    if let Some(translated_frontmatter) = translated_frontmatter {
        f.write_all(format!("{}{}", delimiter, "\n").as_bytes())?;
        f.write_all(translated_frontmatter.as_bytes())?;
        f.write_all(format!("{}{}", delimiter, "\n").as_bytes())?;
    }
    f.write_all(translated_cmark.as_bytes())?;
    f.write_all("\n<!---\n".as_bytes())?;
    f.write_all(cmark_text.as_bytes())?;
    f.write_all("\n-->\n".as_bytes())?;
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

    // translate
    let xml_translated = deepl
        .translate_xml(from_lang, to_lang, formality, &xml)
        .await
        .unwrap();

    // write back to markdown format
    log::trace!("Translated XML: {}\n", &xml_translated);
    let cmark_translated = cmark_xml::cmark_from_xml(&xml_translated, true).unwrap();

    Ok(cmark_translated)
}
