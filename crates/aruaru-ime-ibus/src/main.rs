//! aruaru-IME の IBus エンジン(Linux)。IBus の D-Bus(`org.freedesktop.IBus.Engine`)を話し、変換は常駐エンジン
//! (`aruaru-ime-daemon`、127.0.0.1:4660)に HTTP で頼む。キーを送り、返ってきた確定文字・組み立て中の文字・候補を IBus に反映する。
//! 起動は IBus から(コンポーネントの XML の `<exec>`)。直接起動するときは `--ibus` を付ける。

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("aruaru-ime-ibus は Linux 用です。");
}

#[cfg(target_os = "linux")]
mod linux {
    use std::collections::HashMap;
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::time::Duration;
    use zbus::object_server::SignalContext;
    use zbus::zvariant::{ObjectPath, OwnedObjectPath, StructureBuilder, Value};
    use zbus::{connection, interface, Connection};

    const ENGINE_PORT: u16 = 4660;
    const COMPONENT_NAME: &str = "org.freedesktop.IBus.AruaruIME";

    // ---- X11 キーシム ----
    const KEY_BACKSPACE: u32 = 0xff08;
    const KEY_TAB: u32 = 0xff09;
    const KEY_RETURN: u32 = 0xff0d;
    const KEY_ESCAPE: u32 = 0xff1b;
    const KEY_LEFT: u32 = 0xff51;
    const KEY_UP: u32 = 0xff52;
    const KEY_RIGHT: u32 = 0xff53;
    const KEY_DOWN: u32 = 0xff54;
    const KEY_HENKAN: u32 = 0xff23;
    const KEY_MUHENKAN: u32 = 0xff22;
    const KEY_ZENKAKU: u32 = 0xff2a;
    const KEY_F6: u32 = 0xffc3;
    const KEY_F10: u32 = 0xffc7;
    const SHIFT_MASK: u32 = 1;
    const CONTROL_MASK: u32 = 4;
    const MOD1_MASK: u32 = 8;
    const SUPER_MASK: u32 = 1 << 26;
    const RELEASE_MASK: u32 = 1 << 30;

    /// キーシム → エンジンへ送る Key の JSON(`{"Char":"a"}` / `"Space"` など)。処理しないキーは None。
    pub fn key_json(keyval: u32, state: u32) -> Option<String> {
        if state & (CONTROL_MASK | MOD1_MASK | SUPER_MASK | RELEASE_MASK) != 0 {
            return None;
        }
        let named = match keyval {
            0x20 => Some("Space"),
            KEY_RETURN => Some("Enter"),
            KEY_ESCAPE => Some("Esc"),
            KEY_BACKSPACE => Some("Backspace"),
            KEY_UP => Some("Up"),
            KEY_DOWN => Some("Down"),
            KEY_LEFT => Some("Left"),
            KEY_RIGHT => Some("Right"),
            KEY_TAB => Some("Tab"),
            KEY_HENKAN => Some("Convert"),
            KEY_MUHENKAN => Some("NonConvert"),
            KEY_ZENKAKU => Some("Toggle"),
            _ => None,
        };
        if let Some(n) = named {
            return Some(format!("\"{n}\""));
        }
        if (KEY_F6..=KEY_F10).contains(&keyval) {
            return Some(format!("{{\"Fn\":{}}}", keyval - KEY_F6 + 6));
        }
        if (0x21..=0x7e).contains(&keyval) {
            let c = char::from_u32(keyval)?;
            let esc = if c == '"' || c == '\\' { format!("\\{c}") } else { c.to_string() };
            return Some(format!("{{\"Char\":\"{esc}\"}}"));
        }
        None
    }

    fn http(method: &str, path: &str, body: &str) -> Option<serde_json::Value> {
        let url = format!("http://127.0.0.1:{ENGINE_PORT}{path}");
        let agent = ureq::AgentBuilder::new().timeout(Duration::from_millis(1500)).build();
        let r = if method == "GET" { agent.get(&url).call() } else { agent.post(&url).send_string(body) };
        let text = r.ok()?.into_string().ok()?;
        serde_json::from_str(&text).ok().or_else(|| Some(serde_json::Value::String(text)))
    }

    /// 常駐エンジンが動いていなければ起動する(同じフォルダの aruaru-ime-daemon)。
    fn ensure_daemon() {
        if http("GET", "/health", "").is_some() {
            return;
        }
        if let Ok(exe) = std::env::current_exe() {
            if let Some(dir) = exe.parent() {
                let _ = std::process::Command::new(dir.join("aruaru-ime-daemon")).stdin(std::process::Stdio::null()).stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn();
                for _ in 0..20 {
                    std::thread::sleep(Duration::from_millis(250));
                    if http("GET", "/health", "").is_some() {
                        break;
                    }
                }
            }
        }
    }

    // ---- IBusText / IBusLookupTable の値 ----
    /// `IBusText` = ("IBusText", {}, text, <IBusAttrList>)
    pub fn ibus_text(s: &str) -> Value<'static> {
        let attrs = StructureBuilder::new().add_field("IBusAttrList").add_field(HashMap::<String, Value>::new()).add_field(Vec::<Value>::new()).build();
        let t = StructureBuilder::new().add_field("IBusText").add_field(HashMap::<String, Value>::new()).add_field(s.to_string()).add_field(Value::new(attrs)).build();
        Value::Structure(t)
    }

    /// `IBusLookupTable` = ("IBusLookupTable", {}, page_size, cursor_pos, cursor_visible, round, orientation, [candidates])
    pub fn ibus_lookup_table(cands: &[String], cursor: u32) -> Value<'static> {
        let items: Vec<Value> = cands.iter().map(|c| Value::new(ibus_text(c))).collect();
        let t = StructureBuilder::new()
            .add_field("IBusLookupTable")
            .add_field(HashMap::<String, Value>::new())
            .add_field(10u32)
            .add_field(cursor)
            .add_field(true)
            .add_field(false)
            .add_field(1i32) // vertical
            .add_field(items)
            .build();
        Value::Structure(t)
    }

    pub struct Engine {
        sid: String,
    }

    impl Engine {
        fn new(n: u32) -> Self {
            Engine { sid: format!("ibus-{}-{n}", std::process::id()) }
        }

        /// エンジンの応答(JSON)を IBus に反映する。
        async fn apply(&self, ctxt: &SignalContext<'_>, r: &serde_json::Value) -> zbus::Result<()> {
            if let Some(c) = r["commit"].as_str() {
                if !c.is_empty() {
                    Self::commit_text(ctxt, Value::new(ibus_text(c))).await?;
                }
            }
            let pre = r["preedit"].as_str().unwrap_or("");
            if r["composing"].as_bool().unwrap_or(false) && !pre.is_empty() {
                let n = pre.chars().count() as u32;
                Self::update_preedit_text(ctxt, Value::new(ibus_text(pre)), n, true, 0).await?;
                let cands: Vec<String> = r["cands"].as_array().map(|a| a.iter().filter_map(|x| x.as_str().map(String::from)).collect()).unwrap_or_default();
                if cands.is_empty() {
                    Self::hide_lookup_table(ctxt).await?;
                } else {
                    let cur = r["sel"].as_u64().unwrap_or(0) as u32;
                    Self::update_lookup_table(ctxt, Value::new(ibus_lookup_table(&cands, cur)), true).await?;
                }
            } else {
                Self::hide_preedit_text(ctxt).await?;
                Self::hide_lookup_table(ctxt).await?;
            }
            Ok(())
        }
    }

    #[interface(name = "org.freedesktop.IBus.Engine")]
    impl Engine {
        async fn process_key_event(&self, keyval: u32, _keycode: u32, state: u32, #[zbus(signal_context)] ctxt: SignalContext<'_>) -> bool {
            let Some(k) = key_json(keyval, state) else { return false };
            let body = format!("{{\"sid\":\"{}\",\"key\":{k}}}", self.sid);
            let Some(r) = http("POST", "/key", &body) else { return false };
            let eaten = r["eaten"].as_bool().unwrap_or(false);
            let has_commit = r["commit"].as_str().is_some_and(|c| !c.is_empty());
            if !eaten && !has_commit {
                return false;
            }
            let _ = self.apply(&ctxt, &r).await;
            true
        }

        async fn set_cursor_location(&self, _x: i32, _y: i32, _w: i32, _h: i32) {}
        async fn process_hand_writing_event(&self, _coordinates: Vec<f64>) {}
        async fn cancel_hand_writing(&self, _n_strokes: u32) {}

        async fn focus_in(&self) {}
        async fn focus_out(&self, #[zbus(signal_context)] ctxt: SignalContext<'_>) {
            self.reset(ctxt).await;
        }
        async fn reset(&self, #[zbus(signal_context)] ctxt: SignalContext<'_>) {
            let _ = http("POST", "/session", &format!("{{\"sid\":\"{}\",\"op\":\"cancel\"}}", self.sid));
            let _ = Self::hide_preedit_text(&ctxt).await;
            let _ = Self::hide_lookup_table(&ctxt).await;
        }
        async fn enable(&self) {}
        async fn disable(&self) {}
        async fn page_up(&self) {}
        async fn page_down(&self) {}
        async fn cursor_up(&self) {}
        async fn cursor_down(&self) {}
        async fn candidate_clicked(&self, index: u32, _button: u32, _state: u32, #[zbus(signal_context)] ctxt: SignalContext<'_>) {
            let body = format!("{{\"sid\":\"{}\",\"op\":\"choose\",\"idx\":{index}}}", self.sid);
            if let Some(r) = http("POST", "/session", &body) {
                let _ = self.apply(&ctxt, &r).await;
            }
        }
        async fn set_surrounding_text(&self, _text: Value<'_>, _cursor: u32, _anchor: u32) {}
        async fn set_capabilities(&self, _caps: u32) {}
        async fn property_activate(&self, _name: &str, _state: u32) {}
        async fn property_show(&self, _name: &str) {}
        async fn property_hide(&self, _name: &str) {}
        async fn destroy(&self) {}

        #[zbus(signal)]
        async fn commit_text(ctxt: &SignalContext<'_>, text: Value<'_>) -> zbus::Result<()>;
        #[zbus(signal)]
        async fn update_preedit_text(ctxt: &SignalContext<'_>, text: Value<'_>, cursor_pos: u32, visible: bool, mode: u32) -> zbus::Result<()>;
        #[zbus(signal)]
        async fn hide_preedit_text(ctxt: &SignalContext<'_>) -> zbus::Result<()>;
        #[zbus(signal)]
        async fn update_lookup_table(ctxt: &SignalContext<'_>, table: Value<'_>, visible: bool) -> zbus::Result<()>;
        #[zbus(signal)]
        async fn hide_lookup_table(ctxt: &SignalContext<'_>) -> zbus::Result<()>;
    }

    struct Factory {
        next: AtomicU32,
    }

    #[interface(name = "org.freedesktop.IBus.Factory")]
    impl Factory {
        async fn create_engine(&self, _name: &str, #[zbus(connection)] conn: &Connection) -> zbus::fdo::Result<OwnedObjectPath> {
            let n = self.next.fetch_add(1, Ordering::SeqCst);
            let path = format!("/org/freedesktop/IBus/Engine/{n}");
            conn.object_server().at(path.as_str(), Engine::new(n)).await?;
            Ok(ObjectPath::try_from(path).map_err(zbus::Error::from)?.into())
        }
    }

    /// IBus のバスのアドレス(環境変数 → `ibus address`)。
    fn ibus_address() -> Option<String> {
        if let Ok(a) = std::env::var("IBUS_ADDRESS") {
            if !a.is_empty() {
                return Some(a);
            }
        }
        let out = std::process::Command::new("ibus").arg("address").output().ok()?;
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        (!s.is_empty()).then_some(s)
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        ensure_daemon();
        let addr = ibus_address().ok_or("IBus のバスに接続できません(ibus-daemon が動いていません)")?;
        async_io::block_on(async move {
            let _conn = connection::Builder::address(addr.as_str())?
                .name(COMPONENT_NAME)?
                .serve_at("/org/freedesktop/IBus/Factory", Factory { next: AtomicU32::new(1) })?
                .build()
                .await?;
            eprintln!("aruaru-ime-ibus: ready ({COMPONENT_NAME})");
            futures_lite::future::pending::<()>().await;
            Ok::<(), Box<dyn std::error::Error>>(())
        })
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn keysyms_become_engine_keys() {
            assert_eq!(key_json(b'a' as u32, 0).as_deref(), Some("{\"Char\":\"a\"}"));
            assert_eq!(key_json(0x20, 0).as_deref(), Some("\"Space\""));
            assert_eq!(key_json(KEY_RETURN, 0).as_deref(), Some("\"Enter\""));
            assert_eq!(key_json(KEY_F6, 0).as_deref(), Some("{\"Fn\":6}"));
            assert_eq!(key_json(b'"' as u32, SHIFT_MASK).as_deref(), Some("{\"Char\":\"\\\"\"}"));
            assert_eq!(key_json(b'a' as u32, CONTROL_MASK), None, "Ctrl つきは処理しない");
            assert_eq!(key_json(b'a' as u32, RELEASE_MASK), None, "キーを離したときは処理しない");
        }

        #[test]
        fn ibus_values_have_the_expected_signatures() {
            use zbus::zvariant::Type;
            let _ = ibus_text("あ");
            let v = ibus_text("あ");
            assert_eq!(v.value_signature().to_string(), "(sa{sv}sv)");
            let t = ibus_lookup_table(&["a".into(), "b".into()], 0);
            assert_eq!(t.value_signature().to_string(), "(sa{sv}uubbiav)");
            let _ = <Value as Type>::signature();
        }
    }
}

#[cfg(target_os = "linux")]
fn main() {
    if let Err(e) = linux::run() {
        eprintln!("aruaru-ime-ibus: {e}");
        std::process::exit(1);
    }
}
