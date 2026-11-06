//! Java(`jp.aon.aruaruime.Native`)との接着。Java 側は、ここの関数を呼ぶだけ。
//! jni 0.22: 各関数は `EnvUnowned` を受け取り、`with_env` の中でパニックを捕まえて JVM へ巻き戻さない。

use crate::Engine;
use jni::errors::ThrowRuntimeExAndDefault;
use jni::objects::{JClass, JString};
use jni::{Env, EnvUnowned};
use std::sync::Mutex;

static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

fn get(env: &Env, s: &JString) -> String {
    s.try_to_string(env).unwrap_or_default()
}

fn out<'l>(env: &mut Env<'l>, s: &str) -> jni::errors::Result<JString<'l>> {
    JString::from_str(env, s)
}

/// アプリ起動時に1回。`dir` はアプリのファイル領域(学習した定型文を置く)。
#[no_mangle]
pub extern "system" fn Java_jp_aon_aruaruime_Native_init<'l>(mut env: EnvUnowned<'l>, _c: JClass<'l>, dir: JString<'l>) {
    env.with_env(|env| -> jni::errors::Result<()> {
        let d = get(env, &dir);
        *ENGINE.lock().unwrap_or_else(|e| e.into_inner()) = Some(Engine::new(std::path::Path::new(&d)));
        Ok(())
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

/// 入力先が変わったとき。サイト名(フリマ・オークションでなければ空)を返す。
#[no_mangle]
pub extern "system" fn Java_jp_aon_aruaruime_Native_setField<'l>(mut env: EnvUnowned<'l>, _c: JClass<'l>, package: JString<'l>, secret: bool) -> JString<'l> {
    env.with_env(|env| {
        let p = get(env, &package);
        let r = ENGINE.lock().unwrap_or_else(|e| e.into_inner()).as_mut().map(|e| e.set_field(&p, secret)).unwrap_or_default();
        out(env, &r)
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

/// 画面(WebView のページ)からの呼び出し。通信するので、UI スレッドでは呼ばないこと。
#[no_mangle]
pub extern "system" fn Java_jp_aon_aruaruime_Native_api<'l>(mut env: EnvUnowned<'l>, _c: JClass<'l>, path: JString<'l>, body: JString<'l>, before: JString<'l>) -> JString<'l> {
    env.with_env(|env| {
        let (p, b, bf) = (get(env, &path), get(env, &body), get(env, &before));
        let r = ENGINE.lock().unwrap_or_else(|e| e.into_inner()).as_mut().map(|e| e.api(&p, &b, &bf)).unwrap_or_else(|| "null".to_string());
        out(env, &r)
    })
    .resolve::<ThrowRuntimeExAndDefault>()
}

/// キーボードの画面(HTML)。
#[no_mangle]
pub extern "system" fn Java_jp_aon_aruaruime_Native_page<'l>(mut env: EnvUnowned<'l>, _c: JClass<'l>) -> JString<'l> {
    env.with_env(|env| out(env, &crate::ui::keyboard_html())).resolve::<ThrowRuntimeExAndDefault>()
}
