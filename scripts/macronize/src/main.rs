use clap::{arg, Parser};
use exitfailure::ExitFailure;
use reqwest::Url;
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Name, Predicate};

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// Name of the person to greet
    #[arg(short, long)]
    file: String,
}

impl Args {
    fn new(file: String) -> Self {
        Self { file }
    }
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let args = Args::parse();

    let url = Url::parse("https://alatius.com/macronizer/")?;

    let params = [
        ("textcontent", test_text),
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
