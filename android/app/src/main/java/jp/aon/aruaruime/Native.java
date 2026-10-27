package jp.aon.aruaruime;

/** Rust(libaruaru_ime_android.so)の入口。ロジックは全部 Rust 側にある。 */
final class Native {
    static {
        System.loadLibrary("aruaru_ime_android");
    }

    private Native() {}

    /** アプリのファイル領域を渡す(学習した定型文はここに置く)。 */
    static native void init(String filesDir);

    /** 入力先が変わった。フリマ・オークションならサイト名、そうでなければ空。 */
    static native String setField(String packageName, boolean secret);

    /** 画面からの呼び出し(/key /session /template)。通信するので UI スレッドでは呼ばない。 */
    static native String api(String path, String body, String before);

    /** キーボードの画面(HTML)。 */
    static native String page();
}
