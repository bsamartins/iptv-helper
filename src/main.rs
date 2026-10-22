use std::collections::BTreeSet;
use std::process::ExitCode;

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use clap::Parser;
use serde_json::Value;

const APPDETAIL_URL: &str = "https://purpletvapi.b-cdn.net/appdetail";

// Every field in an `app_dns` entry that can hold a server URL.
const DNS_FIELDS: &[&str] = &[
    "url",
    "live_dns",
    "epg_dns",
    "movie_dns",
    "series_dns",
    "catchup_dns",
    "support_guide_dns",
];

/// List the server domains a Purple TV login code points to.
#[derive(Parser)]
#[command(version)]
struct Args {
    /// Login code entered in the app
    code: String,

    /// Maximum number of base64 layers to peel off the response
    #[arg(long, default_value_t = 10)]
    max_layers: usize,
}

fn main() -> ExitCode {
    let args = Args::parse();

    match run(&args.code, args.max_layers) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::FAILURE
        }
    }
}

fn run(code: &str, max_layers: usize) -> Result<(), Box<dyn std::error::Error>> {
    let url = reqwest::Url::parse_with_params(
        APPDETAIL_URL,
        &[("version", "v2"), ("app_type", "ios"), ("code", code)],
    )?;
    let body = reqwest::blocking::get(url)?
        .error_for_status()?
        .text()?;

    let detail = decode_appdetail(&body, max_layers)?;

    if let Some(name) = detail["app_name"].as_str() {
        println!("App: {name} (mode: {})", detail["app_mode"].as_str().unwrap_or("?"));
    }

    let entries = detail["app_dns"].as_array().ok_or("response has no app_dns list")?;
    let mut domains = BTreeSet::new();

    println!("\nServers:");
    for entry in entries {
        let title = entry["dns_title"].as_str().unwrap_or("(untitled)");
        let url = entry["url"].as_str().unwrap_or("");
        println!("  {title:<15} {url}");

        for field in DNS_FIELDS {
            let Some(value) = entry[*field].as_str().filter(|v| !v.is_empty()) else {
                continue;
            };
            if value != url {
                println!("  {:<15}   {field}: {value}", "");
            }
            domains.insert(value.to_string());
        }
    }

    if let Some(test) = detail["test_server_address"].as_str().filter(|v| !v.is_empty()) {
        println!("  {:<15} {test}", "(test server)");
        domains.insert(test.to_string());
    }

    println!("\nUnique domains ({}):", domains.len());
    for domain in &domains {
        println!("  {domain}");
    }

    Ok(())
}

/// The response is JSON wrapped in several layers of base64; peel them until it parses.
fn decode_appdetail(body: &str, max_layers: usize) -> Result<Value, Box<dyn std::error::Error>> {
    let mut data = body.trim().as_bytes().to_vec();
    for _ in 0..=max_layers {
        if let Ok(json) = serde_json::from_slice::<Value>(&data) {
            return Ok(json);
        }
        data = STANDARD.decode(data.trim_ascii())?;
    }
    Err(format!("still not JSON after {max_layers} base64 layers").into())
}
