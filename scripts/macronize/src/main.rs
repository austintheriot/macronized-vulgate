use std::fs;

use clap::{arg, Parser};
use exitfailure::ExitFailure;
use reqwest::Url;
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Name, Predicate};
use serde_derive::{Deserialize, Serialize};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    path: String,
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
    text: String,
    #[serde(rename(deserialize = "textLatin"))]
    text_latin: String,
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let args = Args::parse();

    let text_to_macronize = fs::read_to_string(args.path)?;

    let testament: Testament = serde_json::from_str(&text_to_macronize)?;

    println!("{:?}", testament);

    return Ok(());

    let url = Url::parse("https://alatius.com/macronizer/")?;

    let params = [
        ("textcontent", text_to_macronize.as_str()),
        ("macronize", "on"),
        ("scan", "0"),
    ];
    let client = reqwest::Client::new();
    let body = client.post(url).form(&params).send().await?.text().await?;

    // Parse the HTML
    let document = Document::from(body.as_str());

    // Find the div with id="selectme"
    if let Some(div) = document
        .find(Name("div").and(Attr("id", "selectme")))
        .next()
    {
        let ambiguous_predicate =
            |node: &Node| node.attr("class").unwrap_or_default().contains("ambig");

        let mut result = String::new();

        // Extract and print the text content
        div.children().into_iter().for_each(|node| {
            let content = node.text().to_string();

            if ambiguous_predicate(&node) {
                result.push_str(&format!("**{}**", content));
            } else {
                result.push_str(&content);
            }
        });

        println!("{:?}", result);
    } else {
        println!("Div with id 'selectme' not found.");
    }

    Ok(())
}
