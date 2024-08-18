use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use clap::{arg, Parser, Subcommand};
use exitfailure::ExitFailure;
use reqwest::Url;
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Name, Predicate};
use serde_derive::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand, Debug)]
enum Command {
    /// macronize a bible chapter directly from local json files
    Json {
        #[arg(short, long)]
        book: String,
        #[arg(short, long)]
        chapter: u32,
    },
    /// specify a filepath to macronize
    Path {
        #[arg(short, long)]
        input_path: String,
        #[arg(short, long)]
        output_path: String,
    },
}

#[derive(Serialize, Deserialize, Debug)]
struct Testament(Vec<Book>);

#[derive(Serialize, Deserialize, Debug)]
struct Book {
    testament: String,
    title: String,
    #[serde(rename(deserialize = "bookNumber"))]
    book_number: u32,
    chapters: Vec<Chapter>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Chapter {
    #[serde(rename(deserialize = "chapterNumber"))]
    chapter_number: u32,
    verses: Vec<Verse>,
}

#[derive(Serialize, Deserialize, Debug)]
struct Verse {
    #[serde(rename(deserialize = "verseNumber"))]
    verse_number: u32,
    text: Option<String>,
    #[serde(rename(deserialize = "textLatin"))]
    text_latin: String,
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let cli = Cli::parse();

    let text_to_macronize = match cli.command {
        Command::Json { ref book, chapter } => get_unmacronized_chapter(book, chapter).await,
        Command::Path { ref input_path, .. } => get_unmacronized_text_from_file(input_path).await,
    }
    .expect("Getting text to macronize");

    let macronized_text = macronize_text(text_to_macronize.as_str())
        .await
        .expect("Macronizing text");

    let output_path_string = match cli.command {
        Command::Json { book, chapter } => {
            format!("../../macronized-md/{}/{}.md", book, chapter)
        }
        Command::Path { output_path, .. } => output_path,
    };

    let output_path = Path::new(&output_path_string);

    // Create the parent directories if they don't exist
    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true) // Create the file if it doesn't exist
        .truncate(true) // Truncate the file if it already exists
        .open(output_path)?;

    // Write the string content to the file
    file.write_all(macronized_text.as_bytes())?;

    println!("File successfully written to {output_path_string}");
    Ok(())
}

async fn get_unmacronized_chapter(book: &str, chapter: u32) -> Result<String, ExitFailure> {
    let old_testament_file_path = Path::new("../../unmacronized-json/old_testament.json");
    let new_testament_file_path = Path::new("../../unmacronized-json/new_testament.json");

    let old_testament_string = fs::read_to_string(old_testament_file_path)?;
    let new_testament_string = fs::read_to_string(new_testament_file_path)?;

    let old_testament: Testament = serde_json::from_str(&old_testament_string)?;
    let new_testament: Testament = serde_json::from_str(&new_testament_string)?;

    let book = old_testament
        .0
        .into_iter()
        .find(|maybe_book| maybe_book.title == book)
        .or_else(|| {
            new_testament
                .0
                .into_iter()
                .find(|maybe_book| maybe_book.title == book)
        })
        .expect("Finding matching book");

    let chapter = book
        .chapters
        .into_iter()
        .find(|maybe_chapter| maybe_chapter.chapter_number == chapter)
        .expect("Finding matching chapter");

    let final_verse_index = chapter.verses.len() - 1;
    let latin_only_text: String = chapter
        .verses
        .into_iter()
        .enumerate()
        .map(|(i, verse)| {
            if i != final_verse_index {
                format!("{} {}\n", verse.verse_number, verse.text_latin)
            } else {
                format!("{} {}", verse.verse_number, verse.text_latin)
            }
        })
        .collect();

    Ok(latin_only_text)
}

async fn get_unmacronized_text_from_file(input_path: &str) -> Result<String, ExitFailure> {
    let file_string = fs::read_to_string(input_path)?;
    Ok(file_string)
}

async fn macronize_text(text_to_macronize: &str) -> Result<String, ExitFailure> {
    let macronizer_url = Url::parse("https://alatius.com/macronizer/")?;

    let client = reqwest::Client::new();
    let body = client
        .post(macronizer_url)
        .form(&[
            ("textcontent", text_to_macronize),
            ("macronize", "on"),
            ("scan", "0"),
        ])
        .send()
        .await?
        .text()
        .await?;

    // Parse the HTML
    let document = Document::from(body.as_str());

    // Find the div with id="selectme"
    let macronized_text = if let Some(div) = document
        .find(Name("div").and(Attr("id", "selectme")))
        .next()
    {
        // find any spans that are marked as ambiguous
        let ambiguous_predicate =
            |node: &Node| node.attr("class").unwrap_or_default().contains("ambig");

        let mut macronized_result = String::new();

        // Extract and print the text content
        div.children().for_each(|node| {
            let string_content = node.text().to_string();
            if ambiguous_predicate(&node) {
                macronized_result.push_str(&format!("**{}**", string_content));
            } else {
                macronized_result.push_str(&string_content);
            }
        });

        Some(macronized_result)
    } else {
        None
    }
    .expect("Parsing API results into String");

    Ok(macronized_text)
}
