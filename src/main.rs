mod config;

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::thread;
use std::time::Duration;
use chrono::Utc;
use reqwest::blocking::Client;
use reqwest::{StatusCode, Url};
use roxmltree::Document;
use config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    let config = Config::parse_from_env();
    println!("{} | {config:?}", Utc::now());

    let Config { host, domain, password, interval } = config;

    let interval = interval.parse::<u64>().expect("configured interval must be valid u64");

    let url = build_url(&host, &domain, &password);

    let client = Client::new();

    loop {
        let res = update(&client, &url);
        println!("{} | {res:?}", Utc::now());
        thread::sleep(Duration::from_secs(interval));
    }
}

fn build_url(host: &str, domain: &str, password: &str) -> Url {
    let mut url = Url::parse("https://dynamicdns.park-your-domain.com/update")
        .expect("supplied base URL should be valid");

    url.query_pairs_mut()
        .append_pair("host", host)
        .append_pair("domain", domain)
        .append_pair("password", password);

   url
}

fn update(client: &Client, url: &Url) -> Result<String, Box<dyn Error>> {
    #[derive(Debug)]
    struct Response {
        description: String,
        number: String,
    }

    impl Display for Response {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            write!(f, "[{number}] {desc}", number = self.number, desc = self.description)
        }
    }

    let res = client.get(url.clone()).send()?;

    if res.status() != StatusCode::OK {
        return Err(format!("non-ok response, got {status}", status = res.status().as_u16()).into());
    }

    let body = res.text()?;

    let document = Document::parse(&body)?;

    let root = document.descendants().find(|n| n.has_tag_name("interface-response"))
        .ok_or("root node missing")?;
    let errors = root.children().find(|n| n.has_tag_name("errors"))
        .map(|errors| errors.children()
            .map(|error| error.text()
                .map(|t| t.to_string())
                .ok_or("text missing from error")
            )
            .collect::<Result<Vec<_>, _>>()
        )
        .ok_or("errors missing from response")??;
    let responses = root.children().find(|n| n.has_tag_name("responses"))
        .map(|responses| responses.children()
            .map(|response| -> Result<Response, &'static str> {
                let description = response.children().find(|n| n.has_tag_name("Description"))
                    .map(|n| n.text().ok_or("response description elem missing text"))
                    .ok_or("response missing description elem")??
                    .to_string();

                let number = response.children().find(|n| n.has_tag_name("ResponseNumber"))
                    .map(|n| n.text().ok_or("response description elem missing text"))
                    .ok_or("response missing description elem")??
                    .to_string();

                Ok(Response {
                    description,
                    number,
                })
            })
            .collect::<Result<Vec<_>, _>>()
        )
        .ok_or("responses missing from response")??
        .into_iter()
        .map(|r| r.to_string()).collect::<Vec<_>>();

    if !errors.is_empty() {
        return Err(format!(
            "errors: {}, responses: {}",
            errors.join(", "),
            responses.join(", ")
        ).into());
    }

    Ok(format!("responses: {responses:?}"))
}
