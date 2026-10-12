//! `aruaru-ime convert <ローマ字> [--before 直前の文] [--url http://127.0.0.1:4600]`
use aruaru_ime_core::{client::Client, client::DEFAULT_URL, convert, Context};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("languages") {
        // aruaru-ime languages [list | apply]
        //   list  : 選択中の言語を表示
        //   apply : 選択中の言語を、Windows の入力方式の一覧(ユーザー単位)に反映する(追加と、選ばれなくなったものの削除)
        use aruaru_ime_core::translit::{self, Lang};
        let st = aruaru_ime_core::settings::Settings::load();
        let chosen = st.selected_languages();
        match args.get(1).map(String::as_str) {
            Some("all") => {
                for (code, ja, native) in aruaru_ime_core::languages::LANGUAGES {
                    let st = if aruaru_ime_core::languages::engine(code).is_some() { "対応 / ready" } else { "準備中 / planned" };
                    println!("{code}	{ja}	{native}	{st}");
                }
            }
            Some("apply") => {
                let clsid = "{B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11}";
                let entries: Vec<String> = translit::ALL
                    .iter()
                    .filter(|l| l.supported())
                    .map(|l| format!("@{{Id=0x{:04X};Guid='{{{:?}}}';On={}}}", l.langid(), uuid_fmt(l.profile_guid()), if chosen.contains(l) { "$true" } else { "$false" }))
                    .collect();
                let script = format!(
                    "$clsid='{clsid}'; $l=Get-WinUserLanguageList; \
                     foreach($e in @({})){{ \
                       $ci=[Globalization.CultureInfo]::GetCultureInfo([int]$e.Id).Name; \
                       $tip=('{{0:X4}}:{{1}}{{2}}' -f $e.Id,$clsid,$e.Guid); \
                       $x=$l | Where-Object LanguageTag -eq $ci; \
                       if($e.On){{ if(-not $x){{ $l.Add($ci); $x=$l | Where-Object LanguageTag -eq $ci }}; if($x.InputMethodTips -notcontains $tip){{ $x.InputMethodTips.Add($tip) }} }} \
                       elseif($x -and ($x.InputMethodTips -contains $tip)){{ [void]$x.InputMethodTips.Remove($tip) }} \
                     }}; Set-WinUserLanguageList $l -Force",
                    entries.join(",")
                );
                let status = std::process::Command::new("powershell").args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script]).status();
                println!("{}", match status { Ok(s) if s.success() => "入力方式の一覧に反映しました(Win+Space で言語を切り替えられます)", _ => "反映できませんでした" });
            }
            _ => {
                for l in &chosen {
                    println!("{:<8} {} / {}{}", l.code(), l.native_name(), l.english_name(), if l.supported() { "" } else { "  (準備中)" });
                }
            }
        }
        let _ = Lang::Ja;
        return;
    }
    if args.first().map(String::as_str) == Some("wish") {
        // 希望用語: aruaru-ime wish add <種類> <名前> | list | remove <種類> <名前>   (種類: manga anime game drama movie programming framework language other)
        use aruaru_ime_core::wish::{self, Category};
        let root = aruaru_ime_core::components::data_dir();
        match (args.get(1).map(String::as_str), args.get(2), args.get(3)) {
            (Some("add"), Some(cat), Some(term)) => {
                let Some(cat) = Category::from_key(cat) else {
                    eprintln!("種類は {} のどれかです", wish::CATEGORIES.iter().map(|c| c.key()).collect::<Vec<_>>().join(" / "));
                    std::process::exit(2);
                };
                let _ = wish::add(&root, term, cat);
                let url = std::env::var("ARUARU_LLM_URL").unwrap_or_else(|_| DEFAULT_URL.to_string());
                let client = Client::new(&url);
                let index = aruaru_ime_core::kanji::Dict::load(&root.join("dict.bin")).ok().or_else(|| aruaru_ime_core::kanji::Dict::load(std::path::Path::new("data/dict.bin")).ok()).map(|d| d.surface_index());
                match wish::research(&client, &root, term, cat, index.as_ref()) {
                    Ok(n) => println!("{term}: 辞書を作りました({n}語)"),
                    Err(e) => println!("{term}: 作れませんでした — {e}"),
                }
            }
            (Some("list"), _, _) => {
                for w in wish::load(&root) {
                    println!("{:<12} {:<24} {:?} {}語 {}", w.category.key(), w.term, w.status, w.words, w.error.unwrap_or_default());
                }
            }
            (Some("remove"), Some(cat), Some(term)) => {
                if let Some(cat) = Category::from_key(cat) {
                    let _ = aruaru_ime_core::userdict::UserDict::load(&root.join("user_dict.json")).remove_source(&wish::source_of(cat, term));
                    println!("{}", wish::remove(&root, term, cat).unwrap_or(false));
                }
            }
            _ => eprintln!("usage: aruaru-ime wish add <種類> <名前> | list | remove <種類> <名前>"),
        }
        return;
    }
    if args.first().map(String::as_str) == Some("frontends") {
        // aruaru-ime frontends [出力先]: 共有のフロントエンドを Windows / macOS / Linux / Android / Web 向けに一括で生成する(RPoem の open-runo-crossui)
        let out = std::path::PathBuf::from(args.get(1).map(String::as_str).unwrap_or("dist/frontends"));
        match open_runo_crossui::generate(&open_runo_crossui::ime_app("http://127.0.0.1:4660", env!("CARGO_PKG_VERSION")), open_runo_crossui::ALL_TARGETS, &out) {
            Ok(f) => println!("{} ファイルを {} に生成しました", f.len(), out.display()),
            Err(e) => eprintln!("生成できませんでした: {e}"),
        }
        return;
    }
    if args.first().map(String::as_str) == Some("stations-install") {
        // aruaru-ime stations-install: 全国の駅名を OpenStreetMap の公開データから取り込む(利用者辞書へ)
        let mut u = aruaru_ime_core::userdict::UserDict::load(&aruaru_ime_core::userdict::UserDict::default_path());
        match aruaru_ime_core::stations::install(&mut u) {
            Ok(n) => println!("{n} 駅を取り込みました(© OpenStreetMap contributors, ODbL)"),
            Err(e) => eprintln!("取り込めませんでした: {e}"),
        }
        return;
    }
    if args.first().map(String::as_str) == Some("postal-install") {
        // aruaru-ime postal-install: 日本郵便の公開データ(utf_ken_all.zip)を取得して、郵便番号 → 住所の辞書を作る
        match aruaru_ime_core::postal::install(&aruaru_ime_core::components::data_dir()) {
            Ok(n) => println!("{n} 件の郵便番号を取り込みました"),
            Err(e) => eprintln!("取得できませんでした: {e}"),
        }
        return;
    }
    if args.first().map(String::as_str) == Some("geo-install") {
        // aruaru-ime geo-install: 世界の郵便番号データ(GeoNames, CC BY 4.0)を取得し、現在地の国も調べる
        match aruaru_ime_core::geo::install(&aruaru_ime_core::components::data_dir()) {
            Ok((n, c)) => println!("{n} か国の郵便番号を取り込みました。現在地の国: {}", c.unwrap_or_else(|| "不明".into())),
            Err(e) => eprintln!("取得できませんでした: {e}"),
        }
        return;
    }
    if args.first().map(String::as_str) == Some("geo") {
        // aruaru-ime geo <郵便番号> [言語コード]: 国名つきの住所の候補を表示する(動作確認用)
        let z = args.get(1).cloned().unwrap_or_default();
        let lang = args.get(2).cloned().unwrap_or_else(|| "ja".into());
        for (i, c) in aruaru_ime_core::geo::candidates(&aruaru_ime_core::components::data_dir(), &z, &lang).iter().enumerate() {
            println!("{}. {c}", i + 1);
        }
        return;
    }
    if args.first().map(String::as_str) == Some("knowledge") {
        // aruaru-ime knowledge add <本文> [タグ…] | search <キーワード> | import <knowledge.json>
        use aruaru_ime_core::knowledge as kb;
        let p = kb::path(&aruaru_ime_core::components::data_dir());
        match args.get(1).map(String::as_str) {
            Some("add") if args.len() > 2 => {
                let tags: Vec<String> = args[3..].to_vec();
                println!("{}", if kb::add(&p, &args[2], &tags, "user").unwrap_or(false) { "追加しました" } else { "追加しませんでした(空・重複)" });
            }
            Some("search") if args.len() > 2 => {
                for i in kb::search(&kb::load(&p), &args[2..].join(" "), 10) {
                    println!("#{} {} [{}]", i.id, i.text, i.tags.join(","));
                }
            }
            Some("import") if args.len() > 2 => {
                println!("{} 件を取り込みました", kb::import(&p, std::path::Path::new(&args[2])).unwrap_or(0));
            }
            _ => eprintln!("使い方: aruaru-ime knowledge add <本文> [タグ…] | search <キーワード> | import <knowledge.json>"),
        }
        return;
    }
    if args.first().map(String::as_str) == Some("backup") {
        // aruaru-ime backup [--dir <フォルダ>] | list | restore <ファイル> [--replace]
        use aruaru_ime_core::backup;
        let root = aruaru_ime_core::components::data_dir();
        let st = aruaru_ime_core::settings::Settings::load();
        let get = |k: &str| args.iter().position(|a| a == k).and_then(|i| args.get(i + 1)).cloned();
        let (dir, in_drive) = backup::backup_dir(get("--dir").or(st.backup_dir.clone()).as_deref());
        match args.get(1).map(String::as_str) {
            Some("list") => {
                println!("保存先: {} {}", dir.display(), if in_drive { "(Google Drive)" } else { "" });
                for (p, n) in backup::list_backups(&dir) {
                    println!("{}  {} KB", p.display(), n / 1024);
                }
            }
            Some("restore") => {
                let Some(file) = args.get(2) else { eprintln!("usage: aruaru-ime backup restore <ファイル> [--replace]"); std::process::exit(2) };
                match backup::read_backup(std::path::Path::new(file)) {
                    Some(b) => println!("{:?}", backup::restore(&b, &root, args.iter().any(|a| a == "--replace"))),
                    None => println!("バックアップを読めません"),
                }
            }
            _ => match backup::write_backup(&root, &dir, true) {
                Ok(Some(p)) => println!("バックアップしました: {} {}", p.display(), if in_drive { "(Google Drive の同期フォルダ)" } else { "(Google Drive のデスクトップ版が見つからないため、ここに保存)" }),
                Ok(None) => println!("バックアップする内容がありません"),
                Err(e) => println!("バックアップできませんでした: {e}"),
            },
        }
        return;
    }
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

/// プロファイル GUID(u128)を、ハイフン区切りの大文字の書式にする。
fn uuid_fmt(g: u128) -> String {
    let h = format!("{g:032X}");
    format!("{}-{}-{}-{}-{}", &h[0..8], &h[8..12], &h[12..16], &h[16..20], &h[20..32])
}

fn serde_json_pretty(r: &aruaru_ime_core::Conversion) -> String {
    let c: Vec<String> = r.candidates.iter().map(|c| format!("  [{}] {}", c.kind, c.text)).collect();
    format!("mode={:?} conf={:.2} ({})\n{}", r.prediction.mode, r.prediction.confidence, r.prediction.reason, c.join("\n"))
}
