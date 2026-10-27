//! キーボードの画面。RPoem(`open-runo-view` の VNode → `open-runo-crossui` の SSR ページ)で生成する。
//! 画面の動き(JS)も Rust の文字列として持ち、Java の `AndroidIme` インターフェース(確定・組み立て中・削除など)だけを呼ぶ。

use open_runo_crossui::{page_html, AppSpec, Bridge};
use open_runo_view::{h, VNode};

const ROWS: [&[&str]; 4] = [
    &["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
    &["q", "w", "e", "r", "t", "y", "u", "i", "o", "p"],
    &["a", "s", "d", "f", "g", "h", "j", "k", "l", "-"],
    &["z", "x", "c", "v", "b", "n", "m", ",", ".", "?"],
];

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
        rows.push(h("div").attr("class", "row").children(keys).build());
    }
    rows.push(
        h("div")
            .attr("class", "row last")
            .child(key("あ", &[("data-a", "mode"), ("id", "mode")], "k act w13"))
            .child(key("⇧", &[("data-a", "shift")], "k act w11"))
            .child(key("/", &[("data-c", "/")], "k"))
            .child(key("+", &[("data-c", "+")], "k"))
            .child(key("空白 / 変換", &[("data-a", "space")], "k act w3"))
            .child(key("⌫", &[("data-a", "bs")], "k act w13"))
            .child(key("⏎", &[("data-a", "enter")], "k act w13"))
            .build(),
    );
    h("div")
        .attr("id", "app")
        .child(h("div").attr("id", "c").build())
        .children(rows)
        .child(h("div").attr("id", "st").child(VNode::Text("aruaru-IME  あ: ローマ字→日本語(サーバーで変換)".into())).build())
        .build()
}

pub const CSS: &str = "html,body{margin:0;background:#1b1d22;color:#fff;font-family:system-ui,sans-serif;-webkit-user-select:none;user-select:none;-webkit-tap-highlight-color:transparent;touch-action:manipulation}\
#app{display:flex;flex-direction:column;padding:2px 2px 4px}\
#c{display:flex;gap:2px;overflow-x:auto;white-space:nowrap;min-height:52px;align-items:center;scrollbar-width:none}#c::-webkit-scrollbar{display:none}\
#c div{padding:8px 14px;font-size:20px;border-radius:6px;flex:0 0 auto}#c .tpl{color:#ffd479}#c .sel{background:#3b4150}\
.row{display:flex;height:48px;margin-top:2px}.row.last{height:52px}\
.k{flex:1 1 0;margin:1px;border:0;border-radius:6px;background:#2c2f36;color:#fff;font-size:18px;padding:0}.k:active{background:#4a9eff}\
.act{background:#3b4150}.w13{flex:1.3 1 0}.w11{flex:1.1 1 0}.w3{flex:3 1 0}\
#st{color:#888;font-size:11px;text-align:center;padding-top:2px}";

/// 画面の動き。`window.AndroidIme`(Java)は、commit / composing / finish / backspace / enter / api を持つ。
pub const SCRIPT: &str = r##"(()=>{
const A=window.AndroidIme;
let composing=false,direct=false,shift=false,seq=0,reading="";
const pend={};
window.__reply=(id,res)=>{const f=pend[id];delete pend[id];if(f)f(res?JSON.parse(res):null)};
const call=(path,body)=>new Promise(ok=>{const id=++seq;pend[id]=ok;A.api(id,path,JSON.stringify(body))});
const cands=document.getElementById("c"),st=document.getElementById("st"),modeBtn=document.getElementById("mode");
const jp=c=>(!direct&&c===",")?"、":(!direct&&c===".")?"。":c;
function show(r){
  cands.innerHTML="";
  if(!r)return;
  (r.cands||[]).slice(0,12).forEach((x,i)=>{const d=document.createElement("div");d.textContent=x;d.dataset.i=i;if(r.sel===i)d.className="sel";cands.appendChild(d)});
  (r.tpl||[]).forEach(t=>{const d=document.createElement("div");d.className="tpl";d.dataset.t=t;d.textContent="📄 "+(t.length>16?t.slice(0,16)+"…":t);cands.appendChild(d)});
}
function apply(r,k){
  if(!r){ if(k&&k.Char)A.commit(jp(k.Char)); return; }
  if(!r.eaten&&r.commit==null){ if(k&&k.Char)A.commit(k.Char); else if(k==="Space")A.commit(" "); return; }
  if(r.commit!=null)A.commit(r.commit);
  composing=!!r.composing;
  if(composing&&r.reading)reading=r.reading;
  if(composing&&r.preedit)A.composing(r.preedit); else A.finish();
  show(composing?r:null);
}
let q=Promise.resolve();
function send(k){ q=q.then(async()=>{ apply(await call("/key",{key:k}),k); }); }
function flush(){ A.finish(); composing=false; show(null); call("/session",{op:"cancel"}); }
function setMode(){ modeBtn.textContent=direct?"A":"あ"; st.textContent=direct?"aruaru-IME  A: 直接入力":"aruaru-IME  あ: ローマ字→日本語(サーバーで変換)"; }
function tap(c){
  if(direct){A.commit(shift?c.toUpperCase():c);return;}
  if(!composing&&/^[0-9]$/.test(c)){A.commit(c);return;}
  send({Char:shift&&/^[a-z]$/.test(c)?c.toUpperCase():c});
  if(shift){shift=false;document.querySelectorAll(".letter").forEach(b=>b.textContent=b.textContent.toLowerCase())}
}
function act(a){
  if(a==="mode"){direct=!direct;flush();setMode();}
  else if(a==="shift"){shift=!shift;document.querySelectorAll(".letter").forEach(b=>b.textContent=shift?b.textContent.toUpperCase():b.textContent.toLowerCase())}
  else if(a==="space"){ if(direct)A.commit(" "); else send("Space"); }
  else if(a==="bs"){ if(!direct&&composing)send("Backspace"); else A.backspace(); }
  else if(a==="enter"){ if(!direct&&composing)send("Enter"); else A.enter(); }
}
document.getElementById("app").addEventListener("pointerdown",async e=>{
  const b=e.target.closest("[data-c],[data-a],[data-i],[data-t]"); if(!b)return;
  e.preventDefault();
  if(b.dataset.c!==undefined)tap(b.dataset.c);
  else if(b.dataset.a)act(b.dataset.a);
  else if(b.dataset.i!==undefined){ q=q.then(async()=>{ apply(await call("/session",{op:"choose",idx:+b.dataset.i}),null); }); }
  else if(b.dataset.t!==undefined){ A.commit(b.dataset.t); A.finish(); composing=false; show(null); call("/template",{text:b.dataset.t,reading}); }
});
window.__flush=flush;
setMode();
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
        // 文字のキー: 数字の行 10 + 3 行 × 10 + 記号(/ +) = 42
        assert_eq!(p.matches("data-c=").count(), 42, "{p}");
    }
}
