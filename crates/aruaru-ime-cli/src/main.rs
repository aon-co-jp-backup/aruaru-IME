//! `aruaru-ime convert <ローマ字> [--before 直前の文] [--url http://127.0.0.1:4600]`
use aruaru_ime_core::{client::Client, client::DEFAULT_URL, convert, Context};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("update") {
        // 手動で更新を確認する: aruaru-ime update [--check]
        let cur = aruaru_ime_core::update::current_version();
        println!("現在の版: v{}.{}.{}", cur.0, cur.1, cur.2);
        if args.iter().any(|a| a == "--check") {
            match aruaru_ime_core::update::check(aruaru_ime_core::update::REPO, cur) {
                Ok(Some(r)) => println!("新しい版があります: {}", r.tag),
                Ok(None) => println!("最新です(または公開された新しい版はまだありません)"),
                Err(e) => println!("確認できませんでした: {e}"),
            }
        } else {
            let root = aruaru_ime_core::components::data_dir().join("app");
            match aruaru_ime_core::update::run_once(aruaru_ime_core::update::REPO, &root) {
                Ok(Some((tag, dir))) => println!("{tag} に更新しました({})。アプリを再起動すると新しい版になります", dir.display()),
                Ok(None) => println!("最新です(または公開された新しい版はまだありません)"),
                Err(e) => println!("更新できませんでした: {e}"),
            }
        }
        return;
    }
    if args.first().map(String::as_str) == Some("dict") {
        use aruaru_ime_core::userdict::UserDict;
        let path = std::env::var("ARUARU_USER_DICT").map(std::path::PathBuf::from).unwrap_or_else(|_| UserDict::default_path());
        let mut u = UserDict::load(&path);
        match (args.get(1).map(String::as_str), args.get(2), args.get(3)) {
            (Some("add"), Some(r), Some(s)) => println!("{}", match u.add(r, s) { Ok(true) => "登録しました".to_string(), Ok(false) => "すでに登録済みです".to_string(), Err(e) => format!("エラー: {e}") }),
            (Some("remove"), Some(r), s) => println!("{} 件削除しました", u.remove(r, s.map(String::as_str)).unwrap_or(0)),
            (Some("list"), _, _) => {
                for e in u.entries() {
                    println!("{}	{}", e.reading, e.surface);
                }
            }
            (Some("export"), _, _) => println!("{}", u.export_tsv()),
            (Some("import"), Some(file), _) => println!("{} 件取り込みました", u.import_tsv(&std::fs::read_to_string(file).unwrap_or_default()).unwrap_or(0)),
            _ => eprintln!("usage: aruaru-ime dict add <読み> <表記> | remove <読み> [表記] | list | export | import <tsvファイル>"),
        }
        return;
    }
    if args.first().map(String::as_str) == Some("ensure-components") {
        use aruaru_ime_core::components::*;
        let root = data_dir();
        for spec in [ARUARU_LLM, OPEN_EASY_WEB] {
            println!("{}: {:?}", spec.name, ensure(&spec, &root, "aruaru-ime", true));
        }
        if is_running(ARUARU_LLM.health_url) {
            println!("qwen: {:?}", ensure_qwen("http://127.0.0.1:4600", "qwen2.5-0.5b-instruct"));
        }
        return;
    }
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
