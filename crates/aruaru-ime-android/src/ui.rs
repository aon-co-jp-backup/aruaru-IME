//! キーボードの画面。RPoem(`open-runo-view` の VNode → `open-runo-crossui` の SSR ページ)で生成する。
//! 画面の動き(JS)も Rust の文字列として持ち、Java の `AndroidIme` インターフェース(確定・組み立て中・削除など)だけを呼ぶ。
//!
//! 画面の配置(スマホ・タブレット共通):
//! - 一番上の段: 左上=入力モードのボタン(あ/カ/A/記)、真ん中=変換候補、右上=「、」「,」(ひらがな・カタカナのとき常に表示)。
//! - 入力モード: ひらがな・カタカナ・アルファベット・記号の4つから選べる(左上のボタンを押すと候補欄に4つ並ぶ)。
//! - 一番下の段: モードのボタンを左上へ移したぶん、「空白 / 変換」を広くした。

use open_runo_crossui::{page_html, AppSpec, Bridge};
use open_runo_view::{h, VNode};

const ROWS: [&[&str]; 4] = [
    &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
    &["a", "s", "d", "f", "g", "h", "j", "k", "l", "-"],
    &["z", "x", "c", "v", "b", "n", "m", ",", ".", "?"],
];

/// 記号モードで、英字の3行の代わりに出す記号(押すとそのまま入る)。
const SYMBOL_ROWS: [&[&str]; 3] = [
    &["！", "？", "「", "」", "（", "）", "・", "ー", "…", "〜"],
    &["@", "#", "$", "%", "&", "*", "_", "=", "~", "^"],
    &[":", ";", "\"", "'", "<", ">", "[", "]", "※", "♪"],
];

/// 入力モード(左上のボタンから選ぶ)。値は (data-m, ボタンの表示)。
pub const MODES: [(&str, &str); 4] = [("hira", "あ"), ("kata", "カ"), ("alpha", "A"), ("sym", "記")];

/// ひらがな・カタカナのとき、候補欄の右上に常に出す句読点。
pub const PUNCT: [&str; 2] = ["、", ","];

fn key(label: &str, attrs: &[(&str, &str)], class: &str) -> VNode {
    let mut e = h("button").attr("type", "button").attr("class", class);
    for (k, v) in attrs {
        e = e.attr(k, v);
    }
    e.child(VNode::Text(label.to_string())).build()
}

/// キーボードの要素の木。
pub fn keyboard_root() -> VNode {
    let mut rows: Vec<VNode> = vec![];
    for (i, r) in ROWS.iter().enumerate() {
        let letters = i > 0;
        let keys = r.iter().map(|k| {
            let is_letter = letters && k.len() == 1 && k.chars().all(|c| c.is_ascii_alphabetic());
            key(k, &[("data-c", k)], if is_letter { "k letter" } else { "k" })
        });
        rows.push(h("div").attr("class", if letters { "row abc" } else { "row" }).children(keys).build());
    }
    for r in SYMBOL_ROWS.iter() {
        let keys = r.iter().map(|k| key(k, &[("data-c", k)], "k"));
        rows.push(h("div").attr("class", "row sym").children(keys).build());
    }
    rows.push(
        h("div")
            .attr("class", "row last")
            .child(key("⇧", &[("data-a", "shift"), ("id", "shift")], "k act w13"))
            .child(key("/", &[("data-c", "/")], "k"))
            .child(key("+", &[("data-c", "+")], "k"))
            .child(key("空白 / 変換", &[("data-a", "space"), ("id", "space")], "k act w4"))
            .child(key("⌫", &[("data-a", "bs")], "k act w13"))
            .child(key("⏎", &[("data-a", "enter")], "k act w13"))
            .build(),
    );
    let punct = h("div")
        .attr("id", "p")
        .children(PUNCT.iter().map(|c| key(c, &[("data-p", c)], "k pk")))
        .build();
    let bar = h("div")
        .attr("id", "bar")
        .child(key("あ", &[("data-a", "mode"), ("id", "mode")], "k act mk"))
        .child(h("div").attr("id", "c").build())
        .child(punct)
        .build();
    h("div")
        .attr("id", "app")
        .child(bar)
        .children(rows)
        .child(h("div").attr("id", "st").child(VNode::Text("aruaru-IME  あ: ひらがな(ローマ字→日本語)".into())).build())
        .build()
}

pub const CSS: &str = "html,body{margin:0;background:#1b1d22;color:#fff;font-family:system-ui,sans-serif;-webkit-user-select:none;user-select:none;-webkit-tap-highlight-color:transparent;touch-action:manipulation}\
html,body,#open-runo-root{height:100%}\
#app{display:flex;flex-direction:column;padding:2px 2px 4px;height:100%;box-sizing:border-box}\
#bar{display:flex;align-items:center;min-height:52px;gap:2px}\
#c{flex:1 1 0;min-width:0;display:flex;gap:2px;overflow-x:auto;white-space:nowrap;align-items:center;scrollbar-width:none}#c::-webkit-scrollbar{display:none}\
#c div{padding:8px 14px;font-size:20px;border-radius:6px;flex:0 0 auto}#c .tpl{color:#ffd479}#c .sel{background:#3b4150}\
#c .md{background:#2c2f36;min-width:28px;text-align:center;padding:8px 10px}#c .md.on{background:#4a9eff}\
#p{display:flex;flex:0 0 auto}#p.off{display:none}\
#bar .mk{flex:0 0 52px;height:44px;font-size:20px}#p .pk{flex:0 0 44px;width:44px;height:44px;font-size:22px}\
#bar{flex:0 0 auto}.row{display:flex;flex:1 1 0;min-height:0;max-height:56px;margin-top:2px}.row.last{flex-grow:1.1}.row.sym{display:none}\
#app.sym .row.sym{display:flex}#app.sym .row.abc{display:none}\
.k{flex:1 1 0;margin:1px;border:0;border-radius:6px;background:#2c2f36;color:#fff;font-size:18px;padding:0}.k:active{background:#4a9eff}\
.act{background:#3b4150}.w13{flex:1.3 1 0}.w4{flex:4 1 0}\
#st{color:#888;font-size:11px;text-align:center;padding-top:2px}\
@media (max-height:280px){#app{padding:1px 2px 2px}#bar{min-height:34px}#bar .mk,#p .pk{height:32px;font-size:18px}#c div{padding:2px 10px;font-size:17px}.row{margin-top:1px}.k{font-size:16px}#st{display:none}}";

/// 画面の動き。`window.AndroidIme`(Java)は、commit / composing / finish / backspace / enter / api を持つ。
pub const SCRIPT: &str = r##"(()=>{
const A=window.AndroidIme;
const MODES=[["hira","あ","ひらがな(ローマ字→日本語)"],["kata","カ","カタカナ(ローマ字→カタカナ)"],["alpha","A","アルファベット(直接入力)"],["sym","記","記号"]];
let mode="hira",menu=false,composing=false,shift=false,seq=0,reading="",conv=false;
const direct=()=>mode==="alpha"||mode==="sym";
const pend={};
window.__reply=(id,res)=>{const f=pend[id];delete pend[id];if(f)f(res?JSON.parse(res):null)};
const call=(path,body)=>new Promise(ok=>{const id=++seq;pend[id]=ok;A.api(id,path,JSON.stringify(body))});
const app=document.getElementById("app"),cands=document.getElementById("c"),st=document.getElementById("st"),modeBtn=document.getElementById("mode"),punct=document.getElementById("p"),space=document.getElementById("space");
// カタカナモードでは、ひらがなをカタカナに直して出す(漢字の候補はそのまま)。
const kata=s=>(mode==="kata"&&s)?s.replace(/[ぁ-ゖ]/g,ch=>String.fromCharCode(ch.charCodeAt(0)+0x60)):s;
const jp=c=>(!direct()&&c===",")?"、":(!direct()&&c===".")?"。":c;
// カタカナモードで「空白 / 変換」をまだ押していない間は、エンジンの自動変換(漢字)を使わず、読みをカタカナで出す。
const kanaOnly=()=>mode==="kata"&&!conv;
function show(r){
  if(menu)return;
  cands.innerHTML="";
  if(!r)return;
  if(kanaOnly()&&reading){const d=document.createElement("div");d.className="sel";d.dataset.k="1";d.textContent=kata(reading);cands.appendChild(d)}
  (r.cands||[]).forEach((x,i)=>{if(kanaOnly()&&kata(x)===kata(reading))return;const d=document.createElement("div");d.textContent=kata(x);d.dataset.i=i;if(r.sel===i)d.className="sel";cands.appendChild(d)});
  (r.tpl||[]).forEach(t=>{const d=document.createElement("div");d.className="tpl";d.dataset.t=t;d.textContent="📄 "+(t.length>16?t.slice(0,16)+"…":t);cands.appendChild(d)});
  // 候補はすべて並べる(横にスクロール)。「変換」を押して選んだ候補が見えるように、そこまでスクロールする。
  const on=cands.querySelector(".sel");if(on)on.scrollIntoView({block:"nearest",inline:"nearest"}); else cands.scrollLeft=0;
}
function apply(r,k){
  if(!r){ if(k&&k.Char)A.commit(jp(k.Char)); return; }
  if(!r.eaten&&r.commit==null){ if(k&&k.Char)A.commit(k.Char); else if(k==="Space")A.commit(" "); return; }
  if(r.commit!=null)A.commit(kata(r.commit));
  composing=!!r.composing;
  if(!composing)conv=false;
  if(composing&&r.reading)reading=r.reading;
  const pre=kanaOnly()?(r.reading||r.preedit):r.preedit;
  if(composing&&pre)A.composing(kata(pre)); else A.finish();
  show(composing?r:null);
}
// カタカナモード: 読みをカタカナのまま確定する(エンジン側の入力は取り消す)。
function commitKana(){ A.commit(kata(reading)); A.finish(); composing=false; conv=false; reading=""; show(null); return call("/session",{op:"cancel"}); }
let q=Promise.resolve();
function send(k){ q=q.then(async()=>{ apply(await call("/key",{key:k}),k); }); }
function flush(){ A.finish(); composing=false; show(null); call("/session",{op:"cancel"}); }
function setShift(v){ shift=v; document.querySelectorAll(".letter").forEach(b=>b.textContent=shift?b.textContent.toUpperCase():b.textContent.toLowerCase()); }
function setMode(m){
  mode=m; menu=false; conv=false;
  const d=MODES.find(x=>x[0]===m);
  modeBtn.textContent=d[1];
  st.textContent="aruaru-IME  "+d[1]+": "+d[2];
  app.classList.toggle("sym",m==="sym");
  punct.classList.toggle("off",direct());
  space.textContent=direct()?"空白":"空白 / 変換";
  if(m==="sym")setShift(false);
  show(null);
}
// 左上のボタン: 押すと候補欄に4つのモードが並び、選んだものに切り替わる(もう一度押すと閉じる)。
function openMenu(){
  if(menu){menu=false;cands.innerHTML="";return;}
  if(composing)flush();
  menu=true; cands.innerHTML="";
  MODES.forEach(([m,l])=>{const d=document.createElement("div");d.className="md"+(m===mode?" on":"");d.dataset.m=m;d.textContent=l;cands.appendChild(d)});
}
function tap(c){
  if(menu){menu=false;cands.innerHTML="";}
  if(direct()){A.commit(shift?c.toUpperCase():c);if(shift)setShift(false);return;}
  if(!composing&&/^[0-9]$/.test(c)){A.commit(c);return;}
  send({Char:shift&&/^[a-z]$/.test(c)?c.toUpperCase():c});
  if(shift)setShift(false);
}
// 右上の「、」「,」: 組み立て中ならその読みを確定してから入れる。
function punctTap(p){ q=q.then(async()=>{ if(composing){ if(kanaOnly())await commitKana(); else apply(await call("/key",{key:"Enter"}),null); } A.commit(p); A.finish(); composing=false; show(null); }); }
function act(a){
  if(a==="mode")openMenu();
  else if(a==="shift"){ if(mode!=="sym")setShift(!shift); }
  else if(a==="space"){ if(direct())A.commit(" "); else { if(composing)conv=true; send("Space"); } }
  else if(a==="bs"){ if(!direct()&&composing)send("Backspace"); else A.backspace(); }
  else if(a==="enter"){ if(!direct()&&composing){ if(kanaOnly())q=q.then(commitKana); else send("Enter"); } else A.enter(); }
}
app.addEventListener("pointerdown",async e=>{
  const b=e.target.closest("[data-c],[data-a],[data-i],[data-t],[data-m],[data-p],[data-k]"); if(!b)return;
  e.preventDefault();
  if(b.dataset.k!==undefined)q=q.then(commitKana);
  else if(b.dataset.m!==undefined)setMode(b.dataset.m);
  else if(b.dataset.p!==undefined)punctTap(b.dataset.p);
  else if(b.dataset.c!==undefined)tap(b.dataset.c);
  else if(b.dataset.a)act(b.dataset.a);
  else if(b.dataset.i!==undefined){ q=q.then(async()=>{ apply(await call("/session",{op:"choose",idx:+b.dataset.i}),null); }); }
  else if(b.dataset.t!==undefined){ A.commit(b.dataset.t); A.finish(); composing=false; show(null); call("/template",{text:b.dataset.t,reading}); }
});
window.__flush=flush;
setMode("hira");
})();"##;

/// キーボードの画面の HTML(WebView に `loadDataWithBaseURL` で読み込む)。
pub fn keyboard_html() -> String {
    let app = AppSpec {
        id: "aruaru-ime-android".into(),
        name: "aruaru-IME".into(),
        version: env!("CARGO_PKG_VERSION").into(),
        engine_url: String::new(),
        bridge: Bridge::default(),
        root: keyboard_root(),
        css: CSS.into(),
        script: Some(SCRIPT.into()),
        daemon: "aruaru-ime-daemon".into(),
    };
    page_html(&app, "")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_has_every_key_and_the_bridge() {
        let p = keyboard_html();
        for k in ["data-c=\"q\"", "data-c=\"a\"", "data-c=\"0\"", "data-a=\"space\"", "data-a=\"mode\"", "data-a=\"shift\"", "data-a=\"bs\"", "data-a=\"enter\"", "id=\"c\"", "id=\"st\""] {
            assert!(p.contains(k), "{k} が無い");
        }
        assert!(p.contains("window.AndroidIme") && p.contains("/template") && p.contains("__reply"));
        // 文字のキー: 数字の行 10 + 英字 3 行 × 10 + 記号 3 行 × 10 + 下の段(/ +) = 72
        assert_eq!(p.matches("data-c=").count(), 72, "{p}");
    }

    #[test]
    fn mode_button_is_top_left_and_punct_top_right() {
        let p = keyboard_html();
        let bar = p.find("id=\"bar\"").expect("bar");
        let mode = p.find("id=\"mode\"").unwrap();
        let cands = p.find("id=\"c\"").unwrap();
        let punct = p.find("id=\"p\"").unwrap();
        let first_row = p.find("class=\"row").unwrap();
        // 上の段の並び: モード → 候補 → 句読点。どれもキーの行より前
        assert!(bar < mode && mode < cands && cands < punct && punct < first_row);
        assert!(p.contains("data-p=\"、\"") && p.contains("data-p=\",\""));
        // 一番下の段にモードのボタンは無い
        let last = p.find("row last").unwrap();
        assert!(!p[last..].contains("data-a=\"mode\""));
        // 4つのモードを選べる
        for m in ["\"hira\"", "\"kata\"", "\"alpha\"", "\"sym\""] {
            assert!(p.contains(m), "{m} が無い");
        }
    }
}
