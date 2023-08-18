mod cmark_xml;
mod deepl;
mod glossary;
mod trans;
mod walkdir;

// re-export
pub use crate::walkdir::new;
pub use cmark_xml::{
    cmark_from_xml, cmark_from_xmldom, read_cmark_with_frontmatter, xml_from_cmark,
    xmldom_from_cmark,
};
pub use deepl::{Deepl, DeeplGlossary, Formality, Language};
pub use glossary::read_glossary;
pub use trans::{translate_cmark, translate_cmark_file, translate_toml};
