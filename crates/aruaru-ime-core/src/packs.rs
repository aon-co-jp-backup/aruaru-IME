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
ごるどらっく\tゴルドラック\nごるどらく\tゴルドラク\nごるどらけ\tゴルドラケ\nごるどらけいたりあごばん\tゴルドラケ(イタリア語版 Goldrake)\nあるこるふらんすごばん\tアルコル(フランス語版 Alcor)\n\
ぐれんだいざーえいごばん\tUFO Robot Grendizer\nゆーろっぱごばんぐれんだいざー\tグレンダイザー(欧州版 Goldorak / Goldrake)\nあくたーらす\tアクターラス\nあるこる\tアルコル\nゔぇにゅじあ\tヴェニュジア\n\
ごるどらっくふらんすごばん\tゴルドラック(フランス語版)\nふらんすごばんごるどらっく\tUFOロボ グレンダイザー(フランス語版 Goldorak)\n\
grendizer\tGrendizer\ngoldorak\tGoldorak\ngoldrake\tGoldrake\nactarus\tActarus\nalcor\tAlcor\nvenusia\tVenusia\nphenicia\tPhenicia\nvega\tVega\nmazinger\tMazinger\n\
ufo\tUFO\n",
    },
    Pack {
        id: "mazinger",
        title: "マジンガーZ",
        category: Category::Anime,
        entries: "\
まじんがーぜっと\tマジンガーZ\nまじんがー\tマジンガー\nまじんが\tマジンガ(イタリア語版 Mazinga)\nかぶとこうじ\t兜甲児\nゆみさやか\t弓さやか\nぼす\tボス\nあしゅらだんしゃく\tあしゅら男爵\nどくたーへる\tドクターヘル\n\
こうしりょくけんきゅうじょ\t光子力研究所\nこうしりょく\t光子力\nぱいるだー\tパイルダー\nろけっとぱんち\tロケットパンチ\nぶれすとふぁいやー\tブレストファイヤー\n\
こうしりょくびーむ\t光子力ビーム\nあふろだいえー\tアフロダイA\nだぶらすえむつー\tダブラスM2\nみけーね\tミケーネ\nぐれーとまじんがー\tグレートマジンガー\n\
てつやつるぎ\t剣鉄也\nぐれんだいざー\tグレンダイザー\nながいごう\t永井豪\n\
mazinger\tMazinger\nmazinga\tMazinga\nmazingerz\tMazingerZ\nmazingaz\tMazingaZ\nkoji\tKoji\nsayaka\tSayaka\nboss\tBoss\ntetsuya\tTetsuya\nafrodite\tAfrodite\n",
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
jeeg\tJeeg\nhiroshi\tHiroshi\nmiwa\tMiwa\nsteel\tSteel\nkotetsujeeg\tKotetsuJeeg\njeegrobot\tJeegRobot\nsteeljeeg\tSteelJeeg\n",
    },
    Pack {
        id: "hunterxhunter",
        title: "ハンターハンター",
        category: Category::Manga,
        entries: "\
はんたーはんたー\tHUNTER×HUNTER\nはんたーはんたー\tハンターハンター\nはんたーしけん\tハンター試験\nはんたーきょうかい\tハンター協会\nはんたーらいせんす\tハンターライセンス\n\
ごんふりーくす\tゴン=フリークス\nごん\tゴン\nきるあぞるでぃっく\tキルア=ゾルディック\nきるあ\tキルア\nくらぴか\tクラピカ\nれおりお\tレオリオ\nひそか\tヒソカ\nいるみ\tイルミ\n\
くろろるしるふる\tクロロ=ルシルフル\nくろろ\tクロロ\nげんえいりょだん\t幻影旅団\nうぼぉぎん\tウボォーギン\nのぶなが\tノブナガ\nふぇいたん\tフェイタン\nしゃるなーく\tシャルナーク\n\
ぱくのだ\tパクノダ\nまち\tマチ\nふらんくりん\tフランクリン\nふぃんくす\tフィンクス\nこるとぴ\tコルトピ\nしずく\tシズク\nぼのれのふ\tボノレノフ\nじんふりーくす\tジン=フリークス\n\
びすけくるーがー\tビスケ=クルーガー\nかいと\tカイト\nごれいぬ\tゴレイヌ\nつぇずげら\tツェズゲラ\nはんぞー\tハンゾー\nぽっくる\tポックル\nぽんず\tポンズ\nとんぱ\tトンパ\n\
あいざっくねてろ\tアイザック・ネテロ\nねてろかいちょう\tネテロ会長\nじゅうにしえん\t十二支ん\nきめらあんと\tキメラアント\nめるえむ\tメルエム\nねふぇるぴとー\tネフェルピトー\nこむぎ\tコムギ\n\
ねん\t念\nてん\t纏\nぜつ\t絶\nれん\t練\nはつ\t発\nよんだいぎょう\t四大行\nぎょう\t凝\nえん\t円\nけん\t堅\nしゅう\t周\nいん\t隠\nこう\t硬\n\
ぐげんかけい\t具現化系\nきょうかけい\t強化系\nほうしゅつけい\t放出系\nへんかけい\t変化系\nそうさけい\t操作系\nとくしつけい\t特質系\n\
よーくしん\tヨークシン\nてんくうとうぎじょう\t天空闘技場\nぐりーどあいらんど\tグリードアイランド\nあんこくたいりく\t暗黒大陸\nとがしよしひろ\t冨樫義博\n\
gon\tGon\nkillua\tKillua\nkurapika\tKurapika\nleorio\tLeorio\nhisoka\tHisoka\nchrollo\tChrollo\nnetero\tNetero\nmeruem\tMeruem\nnen\tNen\n",
    },
    Pack {
        id: "sazaneyes",
        title: "3X3 EYES",
        category: Category::Manga,
        entries: "\
さざんあいず\t3×3 EYES\nさざんあいず\t3X3 EYES\nさざんあいず\tサザンアイズ\nさんじやん\t三只眼\nさんじやんうんから\t三只眼吽迦羅\nうんから\t吽迦羅\nふじいやくも\t藤井八雲\n\
やくも\t八雲\nぱい\tパイ\nたかだゆうぞう\t高田裕三\nきがん\t鬼眼\nきがんおう\t鬼眼王\nうー\t無\n3x3eyes\t3X3EYES\nsazaneyes\tSazanEyes\n",
    },
    Pack {
        id: "hannya",
        title: "般若心経",
        category: Category::Other,
        entries: "\
はんにゃしんぎょう\t般若心経\nまかはんにゃはらみったしんぎょう\t摩訶般若波羅蜜多心経\nはんにゃ\t般若\nはらみった\t波羅蜜多\nはんにゃはらみった\t般若波羅蜜多\nぼさつ\t菩薩\nねはん\t涅槃\n\
かんじざいぼさつ\t観自在菩薩\nぎょうじんはんにゃはらみったじ\t行深般若波羅蜜多時\nしょうけんごうんかいくう\t照見五蘊皆空\nどいっさいくやく\t度一切苦厄\nしゃりし\t舎利子\n\
しきふいくう\t色不異空\nくうふいしき\t空不異色\nしきそくぜくう\t色即是空\nくうそくぜしき\t空即是色\nじゅそうぎょうしき\t受想行識\nやくぶにょぜ\t亦復如是\n\
ぜしょほうくうそう\t是諸法空相\nふしょうふめつ\t不生不滅\nふくふじょう\t不垢不浄\nふぞうふげん\t不増不減\nぜこくうちゅう\t是故空中\nむしき\t無色\nむじゅそうぎょうしき\t無受想行識\n\
むげんにびぜっしんい\t無眼耳鼻舌身意\nむしきしょうこうみそくほう\t無色声香味触法\nむげんかい\t無眼界\nないしむいしきかい\t乃至無意識界\nむむみょう\t無無明\nやくむむみょうじん\t亦無無明尽\n\
ないしむろうし\t乃至無老死\nやくむろうしじん\t亦無老死尽\nむくしゅうめつどう\t無苦集滅道\nむちやくむとく\t無智亦無得\nいむしょとくこ\t以無所得故\nぼだいさった\t菩提薩埵\n\
えはんにゃはらみったこ\t依般若波羅蜜多故\nしんむけいげ\t心無罣礙\nむけいげこ\t無罣礙故\nむうくふ\t無有恐怖\nおんりいっさいてんどうむそう\t遠離一切顛倒夢想\nくぎょうねはん\t究竟涅槃\n\
さんぜしょぶつ\t三世諸仏\nとくあのくたらさんみゃくさんぼだい\t得阿耨多羅三藐三菩提\nこちはんにゃはらみった\t故知般若波羅蜜多\nぜだいじんしゅ\t是大神呪\nぜだいみょうしゅ\t是大明呪\n\
ぜむじょうしゅ\t是無上呪\nぜむとうどうしゅ\t是無等等呪\nのうじょいっさいく\t能除一切苦\nしんじつふこ\t真実不虚\nこせつはんにゃはらみったしゅ\t故説般若波羅蜜多呪\n\
そくせつしゅわつ\t即説呪曰\nぎゃてぎゃてい\t羯諦羯諦\nはらぎゃてい\t波羅羯諦\nはらそうぎゃてい\t波羅僧羯諦\nぼじそわか\t菩提薩婆訶\n\
かんじざいぼさつぎょうじんはんにゃはらみったじ\t観自在菩薩行深般若波羅蜜多時\nしょうけんごうんかいくうどいっさいくやく\t照見五蘊皆空度一切苦厄\n\
ぎゃてぎゃていはらぎゃていはらそうぎゃていぼじそわか\t羯諦羯諦波羅羯諦波羅僧羯諦菩提薩婆訶\n",
    },
];

/// 手で書いたパックと、表から作ったパック(花・国・都市・駅)。
pub fn all() -> &'static [Pack] {
    static ALL: std::sync::OnceLock<Vec<Pack>> = std::sync::OnceLock::new();
    ALL.get_or_init(|| {
        let mut v: Vec<Pack> = PACKS.iter().map(|p| Pack { id: p.id, title: p.title, category: p.category, entries: p.entries }).collect();
        v.extend(table_cache().0.iter().map(|p| Pack { id: p.id, title: p.title, category: p.category, entries: p.entries }));
        v
    })
}

fn table_cache() -> &'static (Vec<Pack>, Vec<crate::packs_i18n::I18n>) {
    static T: std::sync::OnceLock<(Vec<Pack>, Vec<crate::packs_i18n::I18n>)> = std::sync::OnceLock::new();
    T.get_or_init(crate::packs_table::build)
}

pub fn table_i18n() -> &'static [crate::packs_i18n::I18n] {
    &table_cache().1
}

pub fn find(id: &str) -> Option<&'static Pack> {
    all().iter().find(|p| p.id == id)
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
    // まとめて入れる(ファイルへの保存は1回。1件ずつだと、人名辞書のような数千件のパックで、起動に何分もかかる)
    let pairs: Vec<(String, String)> = p.entries.lines().filter_map(|l| l.split_once('\t')).map(|(r, s)| (r.to_string(), s.to_string())).collect();
    let n = user.add_many(&pairs, &src).map_err(|e| e.to_string())?;
    let _ = wish::add(root, p.title, p.category);
    wish::set_status(root, p.title, p.category, Status::Done, n, None);
    // 中国語(簡体・繁体)・韓国語・ヨーロッパ言語の版
    crate::packs_i18n::install(root, p.id);
    Ok(n)
}

/// パックを外す(利用者辞書から出所ごと削除し、用語の記録も消す)。
pub fn uninstall(root: &Path, user: &mut UserDict, p: &Pack) -> Result<usize, String> {
    let n = user.remove_source(&source(p)).map_err(|e| e.to_string())?;
    let _ = wish::remove(root, p.title, p.category);
    crate::packs_i18n::uninstall(root, p.id);
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
        for p in all() {
            let lines: Vec<&str> = p.entries.lines().collect();
            assert!(lines.len() >= 6, "{}: {}", p.id, lines.len());
            for l in lines {
                let (r, s) = l.split_once('\t').unwrap_or_else(|| panic!("{}: {l}", p.id));
                assert!(!r.is_empty() && !s.is_empty() && r.chars().all(|c| matches!(c, 'ぁ'..='ゖ' | 'ー')) || (r.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') && r == s.to_lowercase().replace(' ', "")) || r.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit()), "{}: {l}", p.id);
            }
        }
    }
}
