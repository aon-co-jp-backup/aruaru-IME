//! 組み込みの用語辞書(パック)。右クリックの「用語辞書を選んで入れる」で選ぶと、利用者辞書へ入る(もう一度選ぶと外れる)。
//! 検索して作る用語辞書(`wish`)と同じ出所の形式なので、一覧・ファイル単位の削除・バックアップもそのまま使える。
//! 各行は `読み<TAB>表記`。

use crate::userdict::UserDict;
use crate::wish::{self, Category, Status};
use std::path::Path;

pub struct Pack {
    pub id: &'static str,
    pub title: &'static str,
    pub category: Category,
    pub entries: &'static str,
}

pub const PACKS: &[Pack] = &[
    Pack {
        id: "kimetsu",
        title: "鬼滅の刃",
        category: Category::Anime,
        entries: "\
きめつのやいば\t鬼滅の刃\nきさつたい\t鬼殺隊\nかまどたんじろう\t竈門炭治郎\nかまどねずこ\t竈門禰豆子\nあがつまぜんいつ\t我妻善逸\nはしびらいのすけ\t嘴平伊之助\n\
とみおかぎゆう\t冨岡義勇\nこちょうしのぶ\t胡蝶しのぶ\nれんごくきょうじゅろう\t煉獄杏寿郎\nうずいてんげん\t宇髄天元\nかんろじみつり\t甘露寺蜜璃\n\
ときとうむいちろう\t時透無一郎\nしなずがわさねみ\t不死川実弥\nひめじまぎょうめい\t悲鳴嶼行冥\nいぐろおばない\t伊黒小芭内\nつゆりかなを\t栗花落カナヲ\n\
きぶつじむざん\t鬼舞辻無惨\nうぶやしきかがや\t産屋敷耀哉\nうろこだきさこんじ\t鱗滝左近次\nさびと\t錆兎\nまこも\t真菰\nたまよ\t珠世\nゆしろう\t愈史郎\n\
あかざ\t猗窩座\nどうま\t童磨\nだき\t堕姫\nぎゅうたろう\t妓夫太郎\nえんむ\t魘夢\nこくしぼう\t黒死牟\nなきめ\t鳴女\nぎょっこ\t玉壺\nはんてんぐ\t半天狗\n\
はしら\t柱\nじゅうにきづき\t十二鬼月\nじょうげん\t上弦\nかげん\t下弦\nにちりんとう\t日輪刀\nぜんしゅうちゅう\t全集中\nけっきじゅつ\t血鬼術\n\
みずのこきゅう\t水の呼吸\nほのおのこきゅう\t炎の呼吸\nかみなりのこきゅう\t雷の呼吸\nかぜのこきゅう\t風の呼吸\nいわのこきゅう\t岩の呼吸\n\
むしのこきゅう\t蟲の呼吸\nかすみのこきゅう\t霞の呼吸\nこいのこきゅう\t恋の呼吸\nおとのこきゅう\t音の呼吸\nへびのこきゅう\t蛇の呼吸\n\
はなのこきゅう\t花の呼吸\nひのこきゅう\t日の呼吸\nひのかみかぐら\tヒノカミ神楽\nみなもぎり\t水面斬り\nしらぬい\t不知火\nへきれきいっせん\t霹靂一閃\n\
むげんれっしゃ\t無限列車\nゆうかくへん\t遊郭編\nかたなかじのさと\t刀鍛冶の里\nはしらげいこ\t柱稽古\nなたぐもやま\t那田蜘蛛山\nふじのはな\t藤の花\n\
かすがいがらす\t鎹鴉\nあざ\t痣\nかくは\t赫刃\nすきとおるせかい\t透き通る世界\nちょとつもうしん\t猪突猛進\nごとうげこよみ\t吾峠呼世晴\n",
    },
    Pack {
        id: "onepiece",
        title: "ワンピース",
        category: Category::Manga,
        entries: "\
わんぴーす\tワンピース\nひとつなぎのだいひほう\tひとつなぎの大秘宝\nもんきーでぃーるふぃ\tモンキー・D・ルフィ\nるふぃ\tルフィ\nろろのあぞろ\tロロノア・ゾロ\nぞろ\tゾロ\n\
なみ\tナミ\nうそっぷ\tウソップ\nさんじ\tサンジ\nとにーとにーちょっぱー\tトニートニー・チョッパー\nちょっぱー\tチョッパー\nにころびん\tニコ・ロビン\nろびん\tロビン\n\
ふらんきー\tフランキー\nぶるっく\tブルック\nじんべえ\tジンベエ\nびび\tビビ\nえーす\tエース\nさぼ\tサボ\nしゃんくす\tシャンクス\nしろひげ\t白ひげ\nくろひげ\t黒ひげ\n\
まーしゃるでぃーてぃーち\tマーシャル・D・ティーチ\nごーるでぃーろじゃー\tゴール・D・ロジャー\nかいぐん\t海軍\nかいぞくおう\t海賊王\nむぎわらのいちみ\t麦わらの一味\n\
むぎわらぼうし\t麦わら帽子\nあくまのみ\t悪魔の実\nごむごむのみ\tゴムゴムの実\nはき\t覇気\nぶそうしょくのはき\t武装色の覇気\nけんぶんしょくのはき\t見聞色の覇気\n\
はおうしょくのはき\t覇王色の覇気\nしちぶかい\t七武海\nよんこう\t四皇\nかくめいぐん\t革命軍\nせかいせいふ\t世界政府\nだいかいぞくじだい\t大海賊時代\n\
ぐらんどらいん\tグランドライン\nしんせかい\t新世界\nさうざんどさにーごう\tサウザンド・サニー号\nごーいんぐめりーごう\tゴーイング・メリー号\n\
おだえいいちろう\t尾田栄一郎\nぎやー\tギア\nぎあふぁいぶ\tギア5\nにかのふるーつ\tニカ\nがんまんぐ\tガン・フォール\nえにえすろびー\tエニエス・ロビー\n\
わのくに\tワノ国\nどれすろーざ\tドレスローザ\nしゃぼんでぃしょとう\tシャボンディ諸島\nこうでんしょとう\t空島\nうおーたーせぶん\tウォーターセブン\n",
    },
    // 以下は、日本語の読み・表記に加えて、英語・ヨーロッパ各国での呼び名(Goldorak・Goldrake など)も、ローマ字の読みのまま入れる。
    Pack {
        id: "sailormoon",
        title: "セーラームーン",
        category: Category::Anime,
        entries: "\
せーらーむーん\tセーラームーン\nびしょうじょせんしせーらーむーん\t美少女戦士セーラームーン\nつきのうさぎ\t月野うさぎ\nみずのあみ\t水野亜美\nひのれい\t火野レイ\n\
きのまこと\t木野まこと\nあいのみなこ\t愛野美奈子\nちばまもる\t地場衛\nたきしーどかめん\tタキシード仮面\nるな\tルナ\nあるてみす\tアルテミス\nちびうさ\tちびうさ\n\
せーらーまーきゅりー\tセーラーマーキュリー\nせーらーまーず\tセーラーマーズ\nせーらーじゅぴたー\tセーラージュピター\nせーらーゔぃーなす\tセーラーヴィーナス\n\
せーらーさたーん\tセーラーサターン\nせーらーうらぬす\tセーラーウラヌス\nせーらーねぷちゅーん\tセーラーネプチューン\nせーらーぷるーと\tセーラープルート\n\
たけうちなおこ\t武内直子\nむーんすてぃっく\tムーンスティック\nむーんぷりずむぱわー\tムーンプリズムパワー\nつきにかわっておしおきよ\t月に代わっておしおきよ\n\
だーくきんぐだむ\tダークキングダム\nぶらっくむーん\tブラックムーン\nしるばーみれにあむ\tシルバーミレニアム\nぷりんせす\tプリンセス・セレニティ\n\
usagi\tUsagi\nserena\tSerena\nbunny\tBunny\nluna\tLuna\nartemis\tArtemis\nchibiusa\tChibiusa\nmamoru\tMamoru\ntuxedo\tTuxedo\nmercury\tMercury\n\
venus\tVenus\njupiter\tJupiter\nsaturn\tSaturn\nuranus\tUranus\nneptune\tNeptune\npluto\tPluto\nsailormoon\tSailorMoon\n",
    },
    Pack {
        id: "dragonball",
        title: "ドラゴンボール",
        category: Category::Manga,
        entries: "\
どらごんぼーる\tドラゴンボール\nそんごくう\t孫悟空\nごくう\t悟空\nそんごはん\t孫悟飯\nごはん\t悟飯\nごてん\t悟天\nべじーた\tベジータ\nぶるま\tブルマ\n\
くりりん\tクリリン\nぴっころ\tピッコロ\nかめせんにん\t亀仙人\nふりーざ\tフリーザ\nせる\tセル\nまじんぶう\t魔人ブゥ\nとらんくす\tトランクス\nすーぱーさいやじん\t超サイヤ人\n\
さいやじん\tサイヤ人\nかめはめは\tかめはめ波\nげんきだま\t元気玉\nしぇんろん\t神龍\nきんとうん\t筋斗雲\nにょいぼう\t如意棒\nかいおうけん\t界王拳\n\
とりやまあきら\t鳥山明\nてんかいちぶどうかい\t天下一武道会\nせんず\t仙豆\nなめっくせい\tナメック星\nまふうは\t魔封波\nぶくうじゅつ\t舞空術\n\
ぴらふ\tピラフ\nれっどりぼんぐん\tレッドリボン軍\nぶるーつ\tブルー\nかりん\tカリン\nみすたーさたん\tミスター・サタン\nぼうがい\t悟空\n\
goku\tGoku\ngohan\tGohan\ngoten\tGoten\nvegeta\tVegeta\nbulma\tBulma\nkrillin\tKrillin\npiccolo\tPiccolo\nfrieza\tFrieza\nfreezer\tFreezer\ntrunks\tTrunks\n\
shenron\tShenron\nsaiyan\tSaiyan\nkamehameha\tKamehameha\nnamek\tNamek\nmajinbuu\tMajinBuu\n",
    },
    Pack {
        id: "grendizer",
        title: "UFOロボ グレンダイザー",
        category: Category::Anime,
        entries: "\
ゆーふぉーろぼぐれんだいざー\tUFOロボ グレンダイザー\nぐれんだいざー\tグレンダイザー\nでゅーくふりーど\tデューク・フリード\nうもんだいすけ\t宇門大介\n\
かぶとこうじ\t兜甲児\nまきばひかる\t牧場ひかる\nまじんがーぜっと\tマジンガーZ\nすぺいざー\tスペイザー\nだぶるすぺいざー\tダブルスペイザー\n\
すくりゅーくらっしゃーぱんち\tスクリュークラッシャーパンチ\nだぶるはーけん\tダブルハーケン\nはんどびーむ\tハンドビーム\nすぺーすさんだー\tスペースサンダー\n\
べがせいれんごうぐん\tベガ星連合軍\nべがだいおう\tベガ大王\nふりーどせい\tフリード星\nながいごう\t永井豪\nだいなみっくぷろ\tダイナミックプロ\n\
うちゅうのおうじゃ\t宇宙の王者\nとくしゅぶたい\t特殊部隊\nうちゅうせんかん\t宇宙戦艦\nかがくようさい\t科学要塞研究所\nぐれーとまじんがー\tグレートマジンガー\n\
ごるどらっく\tゴルドラック\nごるどらく\tゴルドラク\nあくたーらす\tアクターラス\nあるこる\tアルコル\nゔぇにゅじあ\tヴェニュジア\n\
ごるどらっくふらんすごばん\tゴルドラック(フランス語版)\nふらんすごばんごるどらっく\tUFOロボ グレンダイザー(フランス語版 Goldorak)\n\
grendizer\tGrendizer\ngoldorak\tGoldorak\ngoldrake\tGoldrake\nactarus\tActarus\nalcor\tAlcor\nvenusia\tVenusia\nphenicia\tPhenicia\nvega\tVega\nmazinger\tMazinger\n\
ufo\tUFO\n",
    },
    Pack {
        id: "mazinger",
        title: "マジンガーZ",
        category: Category::Anime,
        entries: "\
まじんがーぜっと\tマジンガーZ\nかぶとこうじ\t兜甲児\nゆみさやか\t弓さやか\nぼす\tボス\nあしゅらだんしゃく\tあしゅら男爵\nどくたーへる\tドクターヘル\n\
こうしりょくけんきゅうじょ\t光子力研究所\nこうしりょく\t光子力\nぱいるだー\tパイルダー\nろけっとぱんち\tロケットパンチ\nぶれすとふぁいやー\tブレストファイヤー\n\
こうしりょくびーむ\t光子力ビーム\nあふろだいえー\tアフロダイA\nだぶらすえむつー\tダブラスM2\nみけーね\tミケーネ\nぐれーとまじんがー\tグレートマジンガー\n\
てつやつるぎ\t剣鉄也\nぐれんだいざー\tグレンダイザー\nながいごう\t永井豪\n\
mazinger\tMazinger\nmazinga\tMazinga\nkoji\tKoji\nsayaka\tSayaka\nboss\tBoss\ntetsuya\tTetsuya\nafrodite\tAfrodite\n",
    },
    Pack {
        id: "greatmazinger",
        title: "グレートマジンガー",
        category: Category::Anime,
        entries: "\
ぐれーとまじんがー\tグレートマジンガー\nつるぎてつや\t剣鉄也\nつるぎじょうじ\t剣譲二\nぶれーんこんどる\tブレーンコンドル\nさんだーぶれいく\tサンダーブレーク\n\
ぐれーとぶーめらん\tグレートブーメラン\nまじんぱわー\tマジンパワー\nみけーねていこく\tミケーネ帝国\nあくまたいしょう\t暗黒大将軍\nかがくようさいけんきゅうじょ\t科学要塞研究所\n\
まじんがーぜっと\tマジンガーZ\nぐれんだいざー\tグレンダイザー\n\
great\tGreat\ngrandemazinga\tGrande Mazinga\nmazinger\tMazinger\ntetsuya\tTetsuya\n",
    },
    Pack {
        id: "gaiking",
        title: "大空魔竜ガイキング",
        category: Category::Anime,
        entries: "\
だいくうまりゅうがいきんぐ\t大空魔竜ガイキング\nがいきんぐ\tガイキング\nだいくうまりゅう\t大空魔竜\nだりうすだいおう\tダリウス大王\nながいごう\t永井豪\n\
gaiking\tGaiking\ndarius\tDarius\n",
    },
    Pack {
        id: "jeeg",
        title: "鋼鉄ジーグ",
        category: Category::Anime,
        entries: "\
こうてつじーぐ\t鋼鉄ジーグ\nじーぐ\tジーグ\nしばひろし\t司馬宙\nひみこ\t卑弥呼\nやまたいこく\t邪馬台国\nぴっぐしゅーたー\tビッグシューター\nまぐねっとぱわー\tマグネットパワー\nながいごう\t永井豪\n\
jeeg\tJeeg\nhiroshi\tHiroshi\nmiwa\tMiwa\nsteel\tSteel\n",
    },
];

pub fn find(id: &str) -> Option<&'static Pack> {
    PACKS.iter().find(|p| p.id == id)
}

fn source(p: &Pack) -> String {
    wish::source_of(p.category, p.title)
}

pub fn is_installed(root: &Path, p: &Pack) -> bool {
    wish::load(root).iter().any(|w| w.term == p.title && w.category == p.category && w.status == Status::Done)
}

/// パックを利用者辞書に入れる。入れた件数を返す。(すでに入っていれば入れ替え)
pub fn install(root: &Path, user: &mut UserDict, p: &Pack) -> Result<usize, String> {
    let src = source(p);
    user.remove_source(&src).map_err(|e| e.to_string())?;
    let mut n = 0;
    for l in p.entries.lines() {
        if let Some((r, s)) = l.split_once('\t') {
            if user.add_from(r, s, Some(&src)).map_err(|e| e.to_string())? {
                n += 1;
            }
        }
    }
    let _ = wish::add(root, p.title, p.category);
    wish::set_status(root, p.title, p.category, Status::Done, n, None);
    Ok(n)
}

/// パックを外す(利用者辞書から出所ごと削除し、用語の記録も消す)。
pub fn uninstall(root: &Path, user: &mut UserDict, p: &Pack) -> Result<usize, String> {
    let n = user.remove_source(&source(p)).map_err(|e| e.to_string())?;
    let _ = wish::remove(root, p.title, p.category);
    Ok(n)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn install_and_uninstall_a_pack() {
        let d = std::env::temp_dir().join(format!("aruaru-pack-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        let mut u = UserDict::load(&d.join("user_dict.json"));
        let p = find("kimetsu").unwrap();
        assert!(!is_installed(&d, p));
        let n = install(&d, &mut u, p).unwrap();
        assert!(n >= 60, "{n}");
        assert!(is_installed(&d, p));
        assert!(u.entries().iter().any(|e| e.reading == "かまどたんじろう" && e.surface == "竈門炭治郎"));
        assert_eq!(install(&d, &mut u, p).unwrap(), n, "入れ直しは入れ替え");
        assert!(uninstall(&d, &mut u, p).unwrap() >= 60);
        assert!(!is_installed(&d, p));
    }

    #[test]
    fn every_pack_line_is_well_formed() {
        for p in PACKS {
            let lines: Vec<&str> = p.entries.lines().collect();
            assert!(lines.len() >= 6, "{}: {}", p.id, lines.len());
            for l in lines {
                let (r, s) = l.split_once('\t').unwrap_or_else(|| panic!("{}: {l}", p.id));
                assert!(!r.is_empty() && !s.is_empty() && r.chars().all(|c| matches!(c, 'ぁ'..='ゖ' | 'ー')) || (r.chars().all(|c| c.is_ascii_lowercase() || c == '-') && r == s.to_lowercase().replace(' ', "")) || r.chars().all(|c| c.is_ascii_lowercase()), "{}: {l}", p.id);
            }
        }
    }
}
