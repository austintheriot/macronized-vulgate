use exitfailure::ExitFailure;
use reqwest::Url;
use select::document::Document;
use select::node::Node;
use select::predicate::{Attr, Class, Element, Name, Predicate, Text};
use serde_derive::{Deserialize, Serialize};
use std::env;
use std::fmt::format;

#[derive(Serialize, Deserialize, Debug)]
struct CompanyQuote {
    c: f64,
    h: f64,
    l: f64,
    o: f64,
    pc: f64,
    t: i128,
}

impl CompanyQuote {
    async fn get(symbol: &String, api_key: &String) -> Result<Self, ExitFailure> {
        let url = format!(
            "https://finnhub.io/api/v1/quote?symbol={}&token={}",
            symbol, api_key
        );

        let url = Url::parse(&*url)?;
        let res = reqwest::get(url).await?.json::<CompanyQuote>().await?;

        Ok(res)
    }
}

#[tokio::main]
async fn main() -> Result<(), ExitFailure> {
    let test_text = r#"
1 Quare fremuerunt gentes, et populi meditati sunt inania?

2 Astiterunt reges terrae, et principes convenerunt in unum adversus Dominum, et adversus christum ejus.

3 Dirumpamus vincula eorum, et projiciamus a nobis jugum ipsorum.

4 Qui habitat in caelis irridebit eos, et Dominus subsannabit eos.

5 Tunc loquetur ad eos in ira sua, et in furore suo conturbabit eos.

6 Ego autem constitutus sum rex ab eo super Sion, montem sanctum ejus, praedicans praeceptum ejus.

7 Dominus dixit ad me: Filius meus es tu; ego hodie genui te.

8 Postula a me, et dabo tibi gentes haereditatem tuam, et possessionem tuam terminos terrae.

9 Reges eos in virga ferrea, et tamquam vas figuli confringes eos.

10 Et nunc, reges, intelligite; erudimini, qui judicatis terram.

11 Servite Domino in timore, et exsultate ei cum tremore.

12 Apprehendite disciplinam, nequando irascatur Dominus, et pereatis de via justa.

13 Cum exarserit in brevi ira ejus, beati omnes qui confidunt in eo.
"#;

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
