use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

use clap::{arg, Parser};
use exitfailure::ExitFailure;
use reqwest::Url;
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Name, Predicate};
use serde_derive::{Deserialize, Serialize};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    testament: String,
    #[arg(short, long)]
    book: String,
    #[arg(short, long)]
    chapter: u32,
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
    let args = Args::parse();

    let file_path = if args.testament == "new" {
        Path::new("../../unmacronized-json/new_testament.json")
    } else {
        Path::new("../../unmacronized-json/old_testament.json")
    };

    let text_to_macronize = fs::read_to_string(file_path)?;

    let testament: Testament = serde_json::from_str(&text_to_macronize)?;

    let book = testament
        .0
        .into_iter()
        .find(|book| book.title == args.book)
        .expect("Finding matching book");

    let chapter = book
        .chapters
        .into_iter()
        .find(|chapter| chapter.chapter_number == args.chapter)
        .expect("Finding matching chapter");

    let final_verse_index = chapter.verses.len() - 1;
    let latin_only_text: String = chapter
        .verses
        .into_iter()
        .enumerate()
        .map(|(i, verse)| {
            if i != final_verse_index {
                verse.text_latin + "\n"
            } else {
                verse.text_latin
            }
        })
        .collect();

    let macronizer_url = Url::parse("https://alatius.com/macronizer/")?;

    let client = reqwest::Client::new();
    let body = client
        .post(macronizer_url)
        .form(&[
            ("textcontent", latin_only_text.as_str()),
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

    let verses: Vec<&str> = macronized_text.split("\n").collect();

    let verses: Vec<Verse> = verses
        .into_iter()
        .enumerate()
        .map(|(i, verse_text)| Verse {
            text_latin: verse_text.to_string(),
            text: None,
            verse_number: (i as u32) + 1,
        })
        .collect();

    let chapter = Chapter {
        verses,
        chapter_number: args.chapter,
    };

    let path = format!("../../macronized-json/{}/{}.json", args.book, args.chapter);
    let path = Path::new(&path);

    // Create the parent directories if they don't exist
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut file = OpenOptions::new()
        .write(true)
        .create(true) // Create the file if it doesn't exist
        .truncate(true) // Truncate the file if it already exists
        .open(path)?;

    // serialize the chapter as json
    let chapter_json = serde_json::to_string(&chapter)?;

    // Write the string content to the file
    file.write_all(chapter_json.as_bytes())?;

    Ok(())
}
