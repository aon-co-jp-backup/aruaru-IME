//! `aruaru-ime convert <ローマ字> [--before 直前の文] [--url http://127.0.0.1:4600]`
use aruaru_ime_core::{client::Client, client::DEFAULT_URL, convert, Context};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("build-dict") && args.len() == 3 {
        let t = std::time::Instant::now();
        let n = aruaru_ime_core::kanji::build(std::path::Path::new(&args[1]), std::path::Path::new(&args[2])).expect("build");
        println!("{n} entries in {:?}", t.elapsed());
        return;
    }
    if args.first().map(String::as_str) != Some("convert") || args.len() < 2 {
        eprintln!("usage: aruaru-ime convert <romaji> [--before <text>] [--field <kind>] [--url <aruaru-llm>] [--offline]");
        std::process::exit(2);
    }
    let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
    let ctx = Context { before: get("--before").unwrap_or_default(), field_kind: get("--field"), ..Default::default() };
    let client = (!args.iter().any(|a| a == "--offline")).then(|| Client::new(&get("--url").unwrap_or(DEFAULT_URL.into())));
    let client = client.filter(|c| c.alive());
    let dict_path = get("--dict").unwrap_or_else(|| "data/dict.bin".into());
    let dict = aruaru_ime_core::kanji::Dict::load(std::path::Path::new(&dict_path)).ok();
    let r = convert(&args[1], &ctx, client.as_ref(), dict.as_ref());
    println!("{}", serde_json_pretty(&r));
}

fn serde_json_pretty(r: &aruaru_ime_core::Conversion) -> String {
    let c: Vec<String> = r.candidates.iter().map(|c| format!("  [{}] {}", c.kind, c.text)).collect();
    format!("mode={:?} conf={:.2} ({})\n{}", r.prediction.mode, r.prediction.confidence, r.prediction.reason, c.join("\n"))
}
