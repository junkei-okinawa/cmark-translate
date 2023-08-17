mod cmark_xml;
mod deepl;
mod glossary;
mod trans;
mod walkdir;

use std::path::PathBuf;

use clap::{CommandFactory, Parser};
use walkdir::WalkDir;

#[derive(clap::Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<std::path::PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(clap::Subcommand)]
enum Commands {
    /// Translate a CommonMark file
    Translate {
        /// Source language (ISO639-1 2 letter code)
        #[arg(short, long)]
        from: String,
        /// Target language (ISO639-1 2 letter code)
        #[arg(short, long)]
        to: String,
        /// Formality - formal or informal
        #[arg(long)]
        formality: Option<String>,
        /// Input CommonMark file
        input: String,
        /// Output translated CommonMark file
        output: String,
    },
    /// Manage glossaries
    Glossary {
        #[command(subcommand)]
        command: GlossaryCommands,
    },
    /// Show DeepL usage
    Usage,
}

#[derive(clap::Subcommand)]
enum GlossaryCommands {
    /// Register glossary TSV file
    Register {
        /// Glossary name
        #[arg(short, long)]
        name: String,
        /// Source language (ISO639-1 2 letter code)
        #[arg(short, long)]
        from: String,
        /// Target language (ISO639-1 2 letter code)
        #[arg(short, long)]
        to: String,
        /// Input glossary TSV file - First row should contain language codes
        input: std::path::PathBuf,
    },
    /// List registered glossaries
    List,
    /// Delete registered glossary
    Delete {
        /// ID of glossary
        id: String,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> std::io::Result<()> {
    use std::str::FromStr;
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("warn")).init();

    async fn deepl_with_config() -> Result<deepl::Deepl, std::io::Error> {
        // parse commandline
        let cli = Cli::parse();
        if let Some(cfg_file) = cli.config {
            deepl::Deepl::with_config(&cfg_file)
        } else {
            deepl::Deepl::new()
        }
    }

    // Load DeepL config
    let deepl = deepl_with_config().await;
    // parse commandline
    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Translate {
            from,
            to,
            formality,
            input,
            output,
        }) => {
            // Translate CommonMark file
            let lang_from = deepl::Language::from_str(&from)?;
            let lang_to = deepl::Language::from_str(&to)?;
            let formality = formality.map_or(Ok(deepl::Formality::Default), |f| {
                deepl::Formality::from_str(&f)
            })?;

            let input_path = PathBuf::from(&input);
            let input_output = PathBuf::from(&output);
            let is_dir_input = input_path.is_dir();
            let is_dir_output = input_output.extension().is_none();
            if is_dir_input != is_dir_output {
                panic!("Input and output should be both directory or file");
            }
            let files = if is_dir_input {
                // return 用の Vecを生成
                let mut files = Vec::new();
                // 後続処理で使用するOSのPath区切り文字を取得
                let sep = std::path::MAIN_SEPARATOR.to_string();

                // inputディレクトリから再起的に .md ファイルを取得
                let _paths = WalkDir::new(&input)
                    .unwrap()
                    .filter_map(|e| {
                        let file_path = e.unwrap().path();
                        if file_path.extension().is_some() && file_path.extension().unwrap() == "md"
                        {
                            let file_path_string =
                                file_path.into_os_string().into_string().unwrap();

                            // file_path を取得し output 用の file_path を生成する。
                            // path_join_string の先頭文字列がOSの separator文字列だと、
                            // 後続の Path の join で path_join_string だけが有効になってしまうので
                            // 先頭の separator文字列は削除する。
                            let mut path_join_string = file_path_string.replacen(&input, "", 1);
                            path_join_string =
                                if path_join_string.chars().nth(0).unwrap().to_string() == sep {
                                    path_join_string.replacen(&sep, "", 1)
                                } else {
                                    path_join_string
                                };

                            files.push((
                                PathBuf::from(&file_path_string),
                                PathBuf::from(&output).join(path_join_string),
                            ));
                        }
                        Some(())
                    })
                    .collect::<Vec<_>>();
                files
            } else {
                vec![(input_path, input_output.clone())]
            };

            let res = files
                .iter()
                .map(|i| async move {
                    let (input, output) = i;
                    println!("input  : {:?}", input);
                    println!("output : {:?}", output);
                    // Reload DeepL config
                    let deepl = deepl_with_config().await;
                    // run translation
                    let _res = trans::translate_cmark_file(
                        &deepl.unwrap(),
                        lang_from,
                        lang_to,
                        formality,
                        &input,
                        &output,
                    )
                    .await;
                })
                .collect::<Vec<_>>();
            // Wait for all translation tasks
            futures::future::join_all(res).await;
        }
        Some(Commands::Glossary { command }) => {
            // Glossary management
            match command {
                GlossaryCommands::Register {
                    name,
                    from,
                    to,
                    input,
                } => {
                    let from_lang = deepl::Language::from_str(&from)?;
                    let to_lang = deepl::Language::from_str(&to)?;

                    let glossaries = glossary::read_glossary(&name, input).unwrap();

                    let glossary = deepl
                        .unwrap()
                        .register_glossaries(&name, from_lang, to_lang, &glossaries)
                        .await
                        .unwrap();
                    println!(
                        "Total {} entries are registered as ID = {}",
                        glossary.entry_count, glossary.glossary_id
                    );
                }
                GlossaryCommands::List => {
                    // List glossaries
                    let glossaries = deepl.unwrap().list_glossaries().await.unwrap();
                    for glossary in glossaries {
                        println!("{:?}\n", glossary);
                    }
                }
                GlossaryCommands::Delete { id } => {
                    deepl.unwrap().remove_glossary(&id).await.unwrap();
                }
            }
        }
        Some(Commands::Usage) => {
            let used_chars = deepl.unwrap().get_usage().await.unwrap();
            println!("{} characters used.", used_chars);
        }
        _ => {
            // Print help
            Cli::command().print_help()?;
        }
    }

    Ok(())
}
