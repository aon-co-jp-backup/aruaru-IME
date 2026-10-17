//! かな漢字変換: Mozc OSS 辞書(BSD-3-Clause)をコンパイルした単一バイナリを読み込み、
//! 連接コスト付きの最小コスト経路(Viterbi)で変換する。1回の変換はミリ秒単位。
//!
//! バイナリ形式(リトルエンディアン):
//!   magic "AIME1\0\0\0"(8) | n_entries u32 | conn_n u32 | strings_len u32 | unk_id u32
//!   連接コスト i16 × conn_n² (index = 前の rid * conn_n + 次の lid)
//!   エントリ 16B × n: reading_off u32, surface_off u32, rlen u8, slen u8, lid u16, rid u16, cost u16
//!   文字列領域(UTF-8)。エントリは読み(バイト列)昇順。

use std::fs;
use std::path::Path;

const MAGIC: &[u8; 8] = b"AIME1\0\0\0";
const HEADER: usize = 24;
const ENTRY: usize = 16;
const MAX_READING_BYTES: usize = 90;

pub struct Dict {
    data: Vec<u8>,
    n: usize,
    conn_n: usize,
    conn_off: usize,
    ent_off: usize,
    str_off: usize,
    unk_id: u16,
    /// 季節用語・季節の言い回し(組み込み)。
    extra: Vec<Extra>,
}

struct Extra {
    reading: Vec<char>,
    surface: String,
    lid: u16,
    rid: u16,
    cost: i32,
}

#[derive(Clone, Copy)]
struct Ent {
    roff: usize,
    soff: usize,
    rlen: usize,
    slen: usize,
    lid: u16,
    rid: u16,
    cost: u16,
}

fn u32le(b: &[u8], o: usize) -> u32 {
    u32::from_le_bytes(b[o..o + 4].try_into().unwrap())
}
fn u16le(b: &[u8], o: usize) -> u16 {
    u16::from_le_bytes(b[o..o + 2].try_into().unwrap())
}

/// Mozc の `dictionary??.txt` 群・`connection_single_column.txt`・`id.def` から辞書バイナリを作る。
pub fn build(src_dir: &Path, out: &Path) -> std::io::Result<usize> {
    let mut rows: Vec<(String, String, u16, u16, u16)> = Vec::new();
    for i in 0..10 {
        let text = fs::read_to_string(src_dir.join(format!("dictionary{:02}.txt", i)))?;
        for line in text.lines() {
            let mut it = line.split('\t');
            let (Some(r), Some(l), Some(rr), Some(c), Some(s)) = (it.next(), it.next(), it.next(), it.next(), it.next()) else {
                continue;
            };
            let (Ok(l), Ok(rr), Ok(c)) = (l.parse::<u16>(), rr.parse::<u16>(), c.parse::<i32>()) else { continue };
            if r.len() > MAX_READING_BYTES || s.len() > 250 {
                continue;
            }
            rows.push((r.to_string(), s.to_string(), l, rr, c.clamp(0, 65535) as u16));
        }
    }
    rows.sort_by(|a, b| a.0.as_bytes().cmp(b.0.as_bytes()).then(a.4.cmp(&b.4)).then(a.1.cmp(&b.1)));
    rows.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1 && a.2 == b.2 && a.3 == b.3);

    let conn_text = fs::read_to_string(src_dir.join("connection_single_column.txt"))?;
    let mut lines = conn_text.lines();
    let conn_n: usize = lines.next().unwrap_or("0").trim().parse().unwrap_or(0);
    let id_def = fs::read_to_string(src_dir.join("id.def"))?;
    let unk_id: u32 = id_def
        .lines()
        .find_map(|l| {
            let (id, name) = l.split_once(' ')?;
            name.starts_with("名詞,一般,").then(|| id.parse().ok()).flatten()
        })
        .unwrap_or(1);

    let mut strings: Vec<u8> = Vec::new();
    let mut ents: Vec<u8> = Vec::with_capacity(rows.len() * ENTRY);
    let mut last_r: Option<(String, u32)> = None;
    for (r, s, l, rr, c) in &rows {
        let roff = match &last_r {
            Some((lr, off)) if lr == r => *off,
            _ => {
                let off = strings.len() as u32;
                strings.extend_from_slice(r.as_bytes());
                last_r = Some((r.clone(), off));
                off
            }
        };
        let soff = strings.len() as u32;
        strings.extend_from_slice(s.as_bytes());
        ents.extend_from_slice(&roff.to_le_bytes());
        ents.extend_from_slice(&soff.to_le_bytes());
        ents.push(r.len() as u8);
        ents.push(s.len() as u8);
        ents.extend_from_slice(&l.to_le_bytes());
        ents.extend_from_slice(&rr.to_le_bytes());
        ents.extend_from_slice(&c.to_le_bytes());
    }

    let mut buf: Vec<u8> = Vec::new();
    buf.extend_from_slice(MAGIC);
    buf.extend_from_slice(&(rows.len() as u32).to_le_bytes());
    buf.extend_from_slice(&(conn_n as u32).to_le_bytes());
    buf.extend_from_slice(&(strings.len() as u32).to_le_bytes());
    buf.extend_from_slice(&unk_id.to_le_bytes());
    let mut count = 0usize;
    for l in lines.take(conn_n * conn_n) {
        let v: i32 = l.trim().parse().unwrap_or(0);
        buf.extend_from_slice(&(v.clamp(-32768, 32767) as i16).to_le_bytes());
        count += 1;
    }
    if count != conn_n * conn_n {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "connection table size mismatch"));
    }
    buf.extend_from_slice(&ents);
    buf.extend_from_slice(&strings);
    fs::write(out, &buf)?;
    Ok(rows.len())
}

#[derive(Clone)]
struct Node {
    start: usize, // 文字位置
    end: usize,
    lid: u16,
    rid: u16,
    cost: i32,
    surface: String,
    best: i32,
    prev: Option<usize>,
}

impl Dict {
    pub fn load(path: &Path) -> std::io::Result<Self> {
        let data = fs::read(path)?;
        if data.len() < HEADER || &data[..8] != MAGIC {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "not an aruaru-ime dictionary"));
        }
        let n = u32le(&data, 8) as usize;
        let conn_n = u32le(&data, 12) as usize;
        let unk_id = u32le(&data, 20) as u16;
        let conn_off = HEADER;
        let ent_off = conn_off + conn_n * conn_n * 2;
        let str_off = ent_off + n * ENTRY;
        if data.len() < str_off {
            return Err(std::io::Error::new(std::io::ErrorKind::InvalidData, "truncated dictionary"));
        }
        let mut d = Self { data, n, conn_n, conn_off, ent_off, str_off, unk_id, extra: vec![] };
        d.extra = d.build_extra();
        Ok(d)
    }

    /// 読みと表記が一致するエントリ(コスト最小)。
    fn find_ent(&self, reading: &str, surface: &str) -> Option<Ent> {
        let (lo, hi) = self.range(reading.as_bytes());
        (lo..hi).map(|i| self.ent(i)).filter(|e| self.surface(e) == surface).min_by_key(|e| e.cost)
    }

    /// 組み込みの季節用語を、つなぎ方の情報つきの語にする。辞書にある語のつながり(名詞+が+動詞など)は、まとめて1語として安くする。
    fn build_extra(&self) -> Vec<Extra> {
        const BONUS: i32 = 9000;
        let noun = self.find_ent("さくら", "桜");
        let mut out = vec![];
        for (r, w) in crate::seasonal::words() {
            let (lid, rid, cost) = match self.find_ent(&r, &w).or(noun) {
                Some(e) => (e.lid, e.rid, if self.find_ent(&r, &w).is_some() { e.cost as i32 - 2500 } else { 1500 }),
                None => continue,
            };
            out.push(Extra { reading: r.chars().collect(), surface: w, lid, rid, cost });
        }
        for parts in crate::seasonal::phrases() {
            // 辞書に無い語(気功など)は、名詞として扱う
            let ents: Option<Vec<Ent>> = parts.iter().map(|(r, w)| self.find_ent(r, w).or(noun.map(|n| Ent { cost: 5000, ..n }))).collect();
            let Some(ents) = ents else { continue };
            let mut cost: i32 = ents.iter().map(|e| e.cost as i32).sum();
            for w in ents.windows(2) {
                cost += self.conn(w[0].rid, w[1].lid);
            }
            out.push(Extra {
                reading: parts.iter().flat_map(|p| p.0.chars()).collect(),
                surface: parts.iter().map(|p| p.1.as_str()).collect(),
                lid: ents[0].lid,
                rid: ents[ents.len() - 1].rid,
                cost: (cost - BONUS).max(0) / 3,
            });
        }
        out
    }

    pub fn len(&self) -> usize {
        self.n
    }
    pub fn is_empty(&self) -> bool {
        self.n == 0
    }

    fn ent(&self, i: usize) -> Ent {
        let o = self.ent_off + i * ENTRY;
        let d = &self.data;
        Ent {
            roff: u32le(d, o) as usize,
            soff: u32le(d, o + 4) as usize,
            rlen: d[o + 8] as usize,
            slen: d[o + 9] as usize,
            lid: u16le(d, o + 10),
            rid: u16le(d, o + 12),
            cost: u16le(d, o + 14),
        }
    }
    fn reading(&self, e: &Ent) -> &[u8] {
        &self.data[self.str_off + e.roff..self.str_off + e.roff + e.rlen]
    }
    fn surface(&self, e: &Ent) -> String {
        String::from_utf8_lossy(&self.data[self.str_off + e.soff..self.str_off + e.soff + e.slen]).into_owned()
    }
    fn conn(&self, prev_rid: u16, next_lid: u16) -> i32 {
        let (p, q) = (prev_rid as usize, next_lid as usize);
        if p >= self.conn_n || q >= self.conn_n {
            return 10000;
        }
        let o = self.conn_off + (p * self.conn_n + q) * 2;
        i16::from_le_bytes([self.data[o], self.data[o + 1]]) as i32
    }

    /// 読みが完全に一致するエントリ範囲。
    fn range(&self, key: &[u8]) -> (usize, usize) {
        let lo = partition(self.n, |i| self.reading(&self.ent(i)) < key);
        let hi = partition(self.n, |i| self.reading(&self.ent(i)) <= key);
        (lo, hi)
    }

    /// 読み全体に完全一致する表記を、コストの低い順に返す。
    pub fn exact(&self, reading: &str, max: usize) -> Vec<String> {
        let (lo, hi) = self.range(reading.as_bytes());
        let mut v: Vec<(u16, String)> = (lo..hi).map(|i| {
            let e = self.ent(i);
            (e.cost, self.surface(&e))
        }).collect();
        v.sort_by_key(|x| x.0);
        let mut out: Vec<String> = vec![];
        for (_, s) in v {
            if !out.contains(&s) {
                out.push(s);
            }
            if out.len() >= max {
                break;
            }
        }
        out
    }

    /// 表記→(コスト, 読み)の索引(同じ表記は最小コストの読みを採用)。ファイル取り込み時の読み生成用。
    pub fn surface_index(&self) -> std::collections::HashMap<String, (u16, String)> {
        let mut m: std::collections::HashMap<String, (u16, String)> = std::collections::HashMap::with_capacity(self.n);
        for i in 0..self.n {
            let e = self.ent(i);
            let s = self.surface(&e);
            let better = m.get(&s).map_or(true, |(c, _)| e.cost < *c);
            if better {
                m.insert(s, (e.cost, String::from_utf8_lossy(self.reading(&e)).into_owned()));
            }
        }
        m
    }

    /// 予測変換: 読みが `prefix` で始まる(より長い)語を、コストの低い順に返す。(読み, 表記)
    pub fn predict(&self, prefix: &str, max: usize) -> Vec<(String, String)> {
        if prefix.is_empty() {
            return vec![];
        }
        let key = prefix.as_bytes();
        let lo = partition(self.n, |i| self.reading(&self.ent(i)) < key);
        let mut next = key.to_vec();
        if let Some(last) = next.last_mut() {
            *last = last.saturating_add(1);
        }
        let hi = partition(self.n, |i| self.reading(&self.ent(i)) < next.as_slice());
        let mut found: Vec<(u16, String, String)> = Vec::new();
        for i in lo..hi.min(lo + 40000) {
            let e = self.ent(i);
            if e.rlen > key.len() {
                found.push((e.cost, String::from_utf8_lossy(self.reading(&e)).into_owned(), self.surface(&e)));
            }
        }
        found.sort_by_key(|x| x.0);
        let mut out: Vec<(String, String)> = vec![];
        for (_, r, s) in found {
            // ノイズ除去: 1文字だけ伸びた促音・長音終わり、読みをカタカナにしただけの表記、記号を含む表記
            let extra = r.chars().count().saturating_sub(prefix.chars().count());
            let last = s.chars().last().unwrap_or(' ');
            if extra <= 1 && matches!(last, 'っ' | 'ッ' | 'ー' | 'ぁ' | 'ぃ' | 'ぅ' | 'ぇ' | 'ぉ') {
                continue;
            }
            if s.chars().all(|c| matches!(c, 'ァ'..='ヺ' | 'ー')) {
                continue;
            }
            if s.chars().any(|c| c.is_ascii_punctuation() || c.is_ascii_digit()) {
                continue;
            }
            if !out.iter().any(|(_, x)| *x == s) {
                out.push((r, s));
            }
            if out.len() >= max {
                break;
            }
        }
        out
    }

    /// 最小コスト経路での変換結果(文節に分けず連結した文字列)と、(読み, 表記)の区切り。
    pub fn best_path(&self, reading: &str) -> Option<Vec<(String, String)>> {
        self.best_path_limited(reading, usize::MAX)
    }

    /// 辞書の語の長さ(読みの文字数)を `limit` 以下に制限した最小コスト経路。長い1語として載っている語(おはようございます)を、
    /// より短い語(おはよう + ございます)に分けて調べ直すために使う。
    pub fn best_path_limited(&self, reading: &str, limit: usize) -> Option<Vec<(String, String)>> {
        let chars: Vec<char> = reading.chars().collect();
        if chars.is_empty() {
            return None;
        }
        // 文字位置→バイト位置
        let mut boff = vec![0usize; chars.len() + 1];
        for (i, c) in chars.iter().enumerate() {
            boff[i + 1] = boff[i] + c.len_utf8();
        }
        let bytes = reading.as_bytes();
        let mut nodes: Vec<Node> = Vec::new();
        let mut by_start: Vec<Vec<usize>> = vec![vec![]; chars.len() + 1];
        let mut by_end: Vec<Vec<usize>> = vec![vec![]; chars.len() + 1];
        for i in 0..chars.len() {
            let mut found = false;
            for j in i + 1..=chars.len() {
                let key = &bytes[boff[i]..boff[j]];
                if key.len() > MAX_READING_BYTES || j - i > limit {
                    break;
                }
                let (lo, hi) = self.range(key);
                for k in lo..hi {
                    let e = self.ent(k);
                    let idx = nodes.len();
                    nodes.push(Node { start: i, end: j, lid: e.lid, rid: e.rid, cost: e.cost as i32, surface: self.surface(&e), best: i32::MAX, prev: None });
                    by_start[i].push(idx);
                    by_end[j].push(idx);
                    found = true;
                }
            }
            for x in &self.extra {
                let l = x.reading.len();
                if l <= limit && i + l <= chars.len() && chars[i..i + l] == x.reading[..] {
                    let idx = nodes.len();
                    nodes.push(Node { start: i, end: i + l, lid: x.lid, rid: x.rid, cost: x.cost, surface: x.surface.clone(), best: i32::MAX, prev: None });
                    by_start[i].push(idx);
                    by_end[i + l].push(idx);
                    found = true;
                }
            }
            // 未知語: その1文字をそのまま(高コスト)
            let idx = nodes.len();
            nodes.push(Node { start: i, end: i + 1, lid: self.unk_id, rid: self.unk_id, cost: if found { 12000 } else { 9000 }, surface: chars[i].to_string(), best: i32::MAX, prev: None });
            by_start[i].push(idx);
            by_end[i + 1].push(idx);
        }
        // Viterbi(BOS = id 0)
        for i in 0..chars.len() {
            let starts = by_start[i].clone();
            for &ni in &starts {
                let (lid, cost) = (nodes[ni].lid, nodes[ni].cost);
                let mut best = i32::MAX;
                let mut prev = None;
                if i == 0 {
                    best = self.conn(0, lid) + cost;
                } else {
                    for &pi in &by_end[i] {
                        if nodes[pi].best == i32::MAX {
                            continue;
                        }
                        let c = nodes[pi].best + self.conn(nodes[pi].rid, lid) + cost;
                        if c < best {
                            best = c;
                            prev = Some(pi);
                        }
                    }
                }
                nodes[ni].best = best;
                nodes[ni].prev = prev;
            }
        }
        // EOS = id 0
        let mut best = i32::MAX;
        let mut last = None;
        for &pi in &by_end[chars.len()] {
            if nodes[pi].best == i32::MAX {
                continue;
            }
            let c = nodes[pi].best + self.conn(nodes[pi].rid, 0);
            if c < best {
                best = c;
                last = Some(pi);
            }
        }
        let mut path = vec![];
        let mut cur = last;
        while let Some(ni) = cur {
            let n = &nodes[ni];
            path.push((chars[n.start..n.end].iter().collect::<String>(), n.surface.clone()));
            cur = n.prev;
        }
        path.reverse();
        Some(path)
    }

    /// 変換候補(上位 `max` 個)。先頭は最小コスト経路、続いて読み全体に一致する別表記。
    pub fn candidates(&self, reading: &str, max: usize) -> Vec<String> {
        self.candidates_pref(reading, max, false)
    }

    /// 漢字を含む言い換えの変換結果。長い1語(おはようございます)を短い語に分け、3文字以上の語を、漢字を含む別表記に置き換える。
    /// 返す順: 最後の語だけ置き換えたもの(おはよう御座います)→ すべて置き換えたもの(お早う御座います)→ 1語ずつ置き換えたもの。標準の最有力と同じものは除く。
    pub fn kanji_variants(&self, reading: &str) -> Vec<String> {
        let Some(best) = self.best_path(reading) else { return vec![] };
        let standard: String = best.iter().map(|x| x.1.as_str()).collect();
        // 長い語は短い語に分ける
        let mut fine: Vec<(String, String)> = vec![];
        for (r, s) in best {
            let n = r.chars().count();
            if n >= 5 && self.find_ent(&r, &s).is_some() {
                if let Some(sub) = self.best_path_limited(&r, n - 1) {
                    fine.extend(sub);
                    continue;
                }
            }
            fine.push((r, s));
        }
        let has_kanji = |t: &str| t.chars().any(|c| ('一'..='龥').contains(&c));
        // 各語の、漢字を含む別表記(3文字以上の読みの語だけ)
        let alt: Vec<Option<String>> = fine
            .iter()
            .map(|(r, s)| {
                if r.chars().count() >= 3 && !has_kanji(s) {
                    self.exact(r, 8).into_iter().find(|a| has_kanji(a) && a != r)
                } else {
                    None
                }
            })
            .collect();
        let build = |pick: &dyn Fn(usize) -> bool| -> String {
            fine.iter().enumerate().map(|(i, (_, s))| if pick(i) { alt[i].clone().unwrap_or_else(|| s.clone()) } else { s.clone() }).collect()
        };
        let mut out: Vec<String> = vec![];
        let mut push = |t: String, out: &mut Vec<String>| {
            // 数字入りの言い換え(9足)は、読みに数字が無いかぎり出さない
            if t != standard && !out.contains(&t) && !t.chars().any(|c| c.is_ascii_digit()) {
                out.push(t);
            }
        };
        // 分け直しただけで(置き換え無しで)標準と変わるもの。例: よろしくおねがいします → よろしく + お願い + します
        push(build(&|_| false), &mut out);
        if let Some(last) = alt.iter().rposition(|a| a.is_some()) {
            push(build(&|i| i == last), &mut out);
        }
        if alt.iter().filter(|a| a.is_some()).count() > 1 {
            push(build(&|_| true), &mut out);
        }
        for i in (0..alt.len()).rev().filter(|i| alt[*i].is_some()) {
            push(build(&|k| k == i), &mut out);
        }
        out
    }

    /// 「語幹(送り仮名つき)+活用語尾」に分けて、語幹を漢字にした候補。つけて → 付けて・着けて・点けて…。
    /// 語幹は2文字以上で、辞書の別表記が漢字を含み、語幹と同じ最後のかな(送り仮名)で終わるものだけ。
    fn inflected_variants(&self, reading: &str) -> Vec<String> {
        const ENDINGS: [&str; 17] = [
            "ていました", "ています", "ていない", "ていた", "ている", "なかった", "ました", "ません", "たい", "ない", "ます", "たら", "たり", "て", "た", "ずに", "ず",
        ];
        let has_kanji = |t: &str| t.chars().any(|c| ('一'..='龥').contains(&c));
        for end in ENDINGS {
            let Some(stem) = reading.strip_suffix(end) else { continue };
            let Some(last) = stem.chars().last() else { continue };
            if stem.chars().count() < 2 {
                continue;
            }
            let v: Vec<String> = self
                .exact(stem, 12)
                .into_iter()
                .filter(|a| has_kanji(a) && a.chars().last() == Some(last) && a.chars().count() <= stem.chars().count())
                .take(4)
                .map(|a| format!("{a}{end}"))
                .collect();
            if !v.is_empty() {
                return v;
            }
        }
        vec![]
    }

    /// `prefer_kanji` が真なら、漢字を含む言い換え(おはよう御座います)を先頭にし、標準の最有力はその次に出す。
    pub fn candidates_pref(&self, reading: &str, max: usize, prefer_kanji: bool) -> Vec<String> {
        let mut out: Vec<String> = vec![];
        let variants = self.kanji_variants(reading);
        if prefer_kanji {
            if let Some(v) = variants.first() {
                out.push(v.clone());
            }
        }
        let mut digit_best: Option<String> = None;
        if let Some(p) = self.best_path(reading) {
            let t: String = p.iter().map(|x| x.1.as_str()).collect();
            if t.chars().any(|c| c.is_ascii_digit()) && !reading.chars().any(|c| c.is_ascii_digit()) {
                digit_best = Some(t); // 9足 のような数字入りの最有力は、ほかの漢字語の後ろへ
            } else if !out.contains(&t) {
                let all_kana = t == reading;
                out.push(t);
                // 最有力が読みのままのかな(つけて)のとき、語幹+活用語尾に分けて語幹を漢字にした候補(付けて)を先頭に出す
                if all_kana {
                    for (i, v) in self.inflected_variants(reading).into_iter().enumerate() {
                        out.insert(i, v);
                    }
                }
            }
        }
        for v in &variants {
            if !out.contains(v) {
                out.push(v.clone());
            }
        }
        // 読みよりずっと長いかな交じりの語(「この」→「好ましから」のような活用語尾つきの見出し)は、同音の短い語の後ろに回す
        let rl = reading.chars().count();
        let (mut normal, mut odd): (Vec<String>, Vec<String>) = (vec![], vec![]);
        for s in self.exact(reading, max) {
            if out.contains(&s) {
                continue;
            }
            // 数字を含む表記(9足)も、読みだけから漢字語を打つ場面では後ろに回す
            if s.chars().count() > rl * 2 || s.chars().any(|c| c.is_ascii_digit()) {
                odd.push(s);
            } else {
                normal.push(s);
            }
        }
        out.extend(normal);
        out.extend(digit_best);
        out.extend(odd);
        out.truncate(max);
        out
    }
}

/// `pred` が真の間の先頭側の個数(二分探索)。
fn partition(n: usize, pred: impl Fn(usize) -> bool) -> usize {
    let (mut lo, mut hi) = (0, n);
    while lo < hi {
        let mid = (lo + hi) / 2;
        if pred(mid) {
            lo = mid + 1;
        } else {
            hi = mid;
        }
    }
    lo
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 辞書が手元にあるときだけ走る通し試験(`data/dict.bin`)。
    fn dict() -> Option<Dict> {
        let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../data/dict.bin");
        Dict::load(&p).ok()
    }

    #[test]
    fn real_dictionary_inflected_kanji_stem() {
        let Some(d) = dict() else { return };
        for (r, want) in [("つけて", "付けて"), ("つけた", "付けた"), ("つけます", "付けます"), ("つけない", "付けない")] {
            let c = d.candidates(r, 10);
            assert_eq!(c[0], want, "{r}: {c:?}");
        }
        assert!(d.candidates("つけて", 10).contains(&"着けて".to_string()));
    }

    #[test]
    fn real_dictionary_therapy_terms() {
        let Some(d) = dict() else { return };
        for (r, want) in [("きこうりょうほう", "気功療法"), ("おんがくりょうほう", "音楽療法"), ("すいみんりょうほう", "睡眠療法"), ("あろまりょうほう", "アロマ療法"), ("りょうほうをうける", "療法を受ける")] {
            let c = d.candidates(r, 6);
            assert!(c[0].starts_with(&want[..want.char_indices().nth(2).map(|x| x.0).unwrap_or(want.len())]), "{r}: {c:?}");
        }
        assert_eq!(d.candidates("きこうりょうほう", 6)[0], "気功療法");
    }

    #[test]
    fn real_dictionary_seasonal_phrases() {
        let Some(d) = dict() else { return };
        for (r, want) in [
            ("はるにはなのはながさき", "春には菜の花が咲き"),
            ("さくらがさく", "桜が咲く"),
            ("ゆきがふる", "雪が降る"),
            ("あきにはもみじがいろづき", "秋には紅葉が色づき"),
            ("はなびたいかい", "花火大会"),
            ("あきのこうよう", "秋の紅葉"),
            ("そのような", "その様な"),
            ("このように", "この様に"),
            ("はるのさくら", "春の桜"),
        ] {
            let c = d.candidates(r, 5);
            assert_eq!(c[0], want, "{r}: {c:?}");
        }
    }

    #[test]
    fn real_dictionary_kanji_variants_for_formal_endings() {
        let Some(d) = dict() else { return };
        let v = d.kanji_variants("おはようございます");
        assert!(v.contains(&"おはよう御座います".to_string()), "{v:?}");
        assert_eq!(v[0], "おはよう御座います", "最後の語だけ漢字にしたものが先頭");
        let c = d.candidates_pref("おはようございます", 30, true);
        assert_eq!(c[0], "おはよう御座います");
        assert_eq!(c[1], "おはようございます", "標準の変換は次に残る");
        let v2 = d.kanji_variants("よろしくおねがいします");
        assert_eq!(v2[0], "よろしくお願いします", "{v2:?}");
        assert!(v2.contains(&"宜しくお願いします".to_string()));
        let c = d.candidates_pref("ありがとうございました", 30, true);
        assert_eq!(c[0], "ありがとう御座いました");
        // 最初から漢字になる文は、そのまま
        let c = d.candidates_pref("きょうはいいてんきです", 30, true);
        assert_eq!(c[0], "今日はいい天気です");
        // 優先しないとき(従来)は、標準の最有力が先頭
        assert_eq!(d.candidates_pref("おはようございます", 30, false)[0], "おはようございます");
    }

    #[test]
    fn real_dictionary_prediction() {
        let Some(d) = dict() else { return };
        let p = d.predict("ありがとう", 8);
        assert!(p.iter().any(|(_, s)| s == "ありがとうございます"), "{p:?}");
    }

    #[test]
    fn real_dictionary_sentences() {
        let Some(d) = dict() else { return };
        let cases = [
            ("きょうはいいてんきです", "今日はいい天気です"),
            ("わたしはがくせいです", "私は学生です"),
            ("にほんのとうきょうにいく", "日本の東京に行く"),
            ("かいぎのしりょうをおくる", "会議の資料を送る"),
        ];
        for (r, want) in cases {
            let got = d.candidates(r, 5);
            assert_eq!(got[0], want, "{r} -> {got:?}");
        }
    }
}

