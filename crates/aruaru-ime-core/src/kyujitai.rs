//! 旧字体(きゅうじたい): 常用漢字の新字体を、戦前の正字・旧字体に直す。
//! 右クリックメニューから切り替える(設定 `old_forms`)。オンのときは変換結果を旧字体にし、オフのときは候補の末尾に旧字体の別表記を足す。
//! 対応表は代表的な字体の対(新字体 → 旧字体)で、すべての異体字を網羅するものではない。

const PAIRS: &str = "亜亞 悪惡 圧壓 囲圍 為爲 医醫 壱壹 逸逸 隠隱 栄榮 営營 衛衞 駅驛 謁謁 円圓 塩鹽 縁緣 艶艷 応應 欧歐 殴毆 桜櫻 奥奧 横橫 温溫 穏穩 仮假 価價 禍禍 画畫 会會 悔悔 海海 絵繪 壊壞 懐懷 慨慨 概槪 拡擴 殻殼 覚覺 学學 岳嶽 楽樂 渇渴 割割 勧勸 巻卷 寛寬 歓歡 缶罐 観觀 関關 陥陷 艦艦 館館 顔顏 帰歸 気氣 祈祈 器器 偽僞 戯戲 犠犧 旧舊 拠據 挙擧 虚虛 峡峽 挟挾 狭狹 郷鄕 響響 暁曉 勤勤 謹謹 区區 駆驅 愚愚 勲勳 薫薰 径徑 恵惠 啓啓 掲揭 渓溪 経經 蛍螢 軽輕 継繼 鶏鷄 芸藝 撃擊 欠缺 研硏 県縣 倹儉 剣劍 険險 圏圈 検檢 献獻 絹絹 権權 顕顯 験驗 厳嚴 広廣 恒恆 鉱鑛 号號 国國 黒黑 穀穀 砕碎 済濟 斎齋 剤劑 殺殺 雑雜 参參 惨慘 桟棧 蚕蠶 賛贊 残殘 糸絲 祉祉 視視 歯齒 嗣嗣 児兒 辞辭 湿濕 実實 舎舍 写寫 煮煮 社社 者者 釈釋 寿壽 収收 臭臭 従從 渋澁 獣獸 縦縱 粛肅 処處 暑暑 署署 緒緖 諸諸 叙敍 将將 傷傷 渉涉 焼燒 証證 奨獎 条條 状狀 乗乘 浄淨 剰剩 畳疊 縄繩 壌壤 嬢孃 譲讓 醸釀 触觸 嘱囑 神神 真眞 寝寢 慎愼 尽盡 図圖 粋粹 酔醉 穂穗 随隨 髄髓 枢樞 数數 瀬瀨 声聲 斉齊 静靜 税稅 夕夕 积積 摂攝 節節 専專 浅淺 戦戰 践踐 銭錢 潜潛 繊纖 禅禪 双雙 壮壯 争爭 荘莊 捜搜 挿插 巣巢 窓窗 総總 聡聰 荘莊 蔵藏 臓臟 属屬 続續 堕墮 対對 体體 帯帶 滞滯 台臺 滝瀧 択擇 沢澤 担擔 単單 胆膽 嘆嘆 団團 断斷 弾彈 遅遲 痴癡 虫蟲 昼晝 鋳鑄 駐駐 著著 庁廳 聴聽 懲懲 勅敕 鎮鎭 塚塚 逓遞 鉄鐵 転轉 点點 伝傳 党黨 盗盜 稲稻 闘鬪 徳德 独獨 読讀 突突 届屆 縄繩 弐貳 悩惱 脳腦 覇霸 拝拜 廃廢 売賣 梅梅 麦麥 発發 髪髮 抜拔 繁繁 晩晚 蛮蠻 卑卑 秘祕 碑碑 浜濱 瓶甁 払拂 仏佛 併倂 並竝 変變 辺邊 弁辯 勉勉 歩步 宝寶 豊豐 褒襃 墨墨 翻飜 毎每 万萬 満滿 免免 麺麵 黙默 弥彌 訳譯 薬藥 与與 予豫 余餘 誉譽 揺搖 様樣 謡謠 来來 頼賴 乱亂 覧覽 欄欄 竜龍 隆隆 虜虜 両兩 猟獵 緑綠 涙淚 塁壘 類類 礼禮 励勵 戻戾 霊靈 齢齡 暦曆 歴歷 恋戀 練練 連連 廉廉 炉爐 労勞 郎郞 朗朗 楼樓 録錄 湾灣 関關 当當 党黨 読讀 糖糖 蔵藏 壱壹 参參 弐貳";

fn table() -> std::collections::HashMap<char, char> {
    PAIRS
        .split_whitespace()
        .filter_map(|p| {
            let mut it = p.chars();
            Some((it.next()?, it.next()?))
        })
        .collect()
}

/// 新字体を旧字体に直す。対応表に無い文字はそのまま。
pub fn to_old(s: &str) -> String {
    let t = table();
    s.chars().map(|c| t.get(&c).copied().unwrap_or(c)).collect()
}

/// 旧字体を新字体に戻す(逆引き)。
pub fn to_new(s: &str) -> String {
    let rev: std::collections::HashMap<char, char> = table().into_iter().map(|(a, b)| (b, a)).collect();
    s.chars().map(|c| rev.get(&c).copied().unwrap_or(c)).collect()
}

/// 旧字体に直したときに変わるか。
pub fn has_old_form(s: &str) -> bool {
    to_old(s) != s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_common_words() {
        assert_eq!(to_old("国"), "國");
        assert_eq!(to_old("学校"), "學校");
        assert_eq!(to_old("日本国憲法"), "日本國憲法");
        assert_eq!(to_old("桜"), "櫻");
        assert_eq!(to_old("竜"), "龍");
        assert_eq!(to_old("体"), "體");
        assert_eq!(to_old("気持ち"), "氣持ち");
        assert_eq!(to_old("広島"), "廣島");
        assert_eq!(to_old("ひらがな ABC"), "ひらがな ABC", "対応表に無い文字・英数字は変えない");
    }

    #[test]
    fn round_trips() {
        for s in ["国", "学校", "桜", "竜", "体", "気持ち", "広島", "東京都"] {
            assert_eq!(to_new(&to_old(s)), s, "{s}");
        }
        assert!(has_old_form("国") && !has_old_form("東京"));
    }

    #[test]
    fn table_is_well_formed() {
        for p in PAIRS.split_whitespace() {
            assert_eq!(p.chars().count(), 2, "{p}");
        }
    }
}
