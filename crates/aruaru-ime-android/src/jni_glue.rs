//! Java(`jp.aon.aruaruime.Native`)との接着。Java 側は、ここの関数を呼ぶだけ。

use crate::Engine;
use jni::objects::{JClass, JString};
use jni::sys::{jboolean, jstring};
use jni::JNIEnv;
use std::sync::Mutex;

static ENGINE: Mutex<Option<Engine>> = Mutex::new(None);

fn get(env: &mut JNIEnv, s: &JString) -> String {
    env.get_string(s).map(|x| x.into()).unwrap_or_default()
}

fn out(env: &JNIEnv, s: &str) -> jstring {
    env.new_string(s).map(|x| x.into_raw()).unwrap_or(std::ptr::null_mut())
}

/// アプリ起動時に1回。`dir` はアプリのファイル領域(学習した定型文を置く)。
#[no_mangle]
pub extern "system" fn Java_jp_aon_aruaruime_Native_init(mut env: JNIEnv, _c: JClass, dir: JString) {
    let d = get(&mut env, &dir);
    *ENGINE.lock().unwrap_or_else(|e| e.into_inner()) = Some(Engine::new(std::path::Path::new(&d)));
}

/// 入力先が変わったとき。サイト名(フリマ・オークションでなければ空)を返す。
#[no_mangle]
pub extern "system" fn Java_jp_aon_aruaruime_Native_setField(mut env: JNIEnv, _c: JClass, package: JString, secret: jboolean) -> jstring {
    let p = get(&mut env, &package);
    let r = ENGINE.lock().unwrap_or_else(|e| e.into_inner()).as_mut().map(|e| e.set_field(&p, secret != 0)).unwrap_or_default();
    out(&env, &r)
}

/// 画面(WebView のページ)からの呼び出し。通信するので、UI スレッドでは呼ばないこと。
#[no_mangle]
pub extern "system" fn Java_jp_aon_aruaruime_Native_api(mut env: JNIEnv, _c: JClass, path: JString, body: JString, before: JString) -> jstring {
    let (p, b, bf) = (get(&mut env, &path), get(&mut env, &body), get(&mut env, &before));
    let r = ENGINE.lock().unwrap_or_else(|e| e.into_inner()).as_mut().map(|e| e.api(&p, &b, &bf)).unwrap_or_else(|| "null".to_string());
    out(&env, &r)
}

/// キーボードの画面(HTML)。
#[no_mangle]
pub extern "system" fn Java_jp_aon_aruaruime_Native_page(env: JNIEnv, _c: JClass) -> jstring {
    out(&env, &crate::ui::keyboard_html())
}
