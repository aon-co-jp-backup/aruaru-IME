package jp.aon.aruaruime;

import android.annotation.SuppressLint;
import android.inputmethodservice.InputMethodService;
import android.os.Handler;
import android.os.Looper;
import android.text.InputType;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.EditorInfo;
import android.view.inputmethod.InputConnection;
import android.webkit.JavascriptInterface;
import android.webkit.WebView;

import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * aruaru-IME のキーボード。画面もロジックも Rust(RPoem で生成した HTML + libaruaru_ime_android.so)にある。
 * この Java は、WebView を載せて、入力欄の操作(確定・組み立て中・削除)を中継する接着だけ。
 */
public class AruaruImeService extends InputMethodService {
    private final ExecutorService net = Executors.newSingleThreadExecutor();
    private final Handler ui = new Handler(Looper.getMainLooper());
    private WebView web;

    @SuppressLint({"SetJavaScriptEnabled", "AddJavascriptInterface"})
    @Override
    public View onCreateInputView() {
        Native.init(getFilesDir().getAbsolutePath());
        web = new WebView(this);
        web.getSettings().setJavaScriptEnabled(true);
        web.setBackgroundColor(0xFF1B1D22);
        web.addJavascriptInterface(new Bridge(), "AndroidIme");
        web.loadDataWithBaseURL("https://localhost/", Native.page(), "text/html", "UTF-8", null);
        web.setLayoutParams(new android.view.ViewGroup.LayoutParams(-1, (int) (300 * getResources().getDisplayMetrics().density)));
        return web;
    }

    /** ページ(JS)から呼ばれる。 */
    private final class Bridge {
        @JavascriptInterface
        public void commit(String s) {
            ui.post(() -> {
                InputConnection ic = getCurrentInputConnection();
                if (ic != null) ic.commitText(s, 1);
            });
        }

        @JavascriptInterface
        public void composing(String s) {
            ui.post(() -> {
                InputConnection ic = getCurrentInputConnection();
                if (ic != null) ic.setComposingText(s, 1);
            });
        }

        @JavascriptInterface
        public void finish() {
            ui.post(() -> {
                InputConnection ic = getCurrentInputConnection();
                if (ic != null) ic.finishComposingText();
            });
        }

        @JavascriptInterface
        public void backspace() {
            ui.post(() -> sendDownUpKeyEvents(KeyEvent.KEYCODE_DEL));
        }

        @JavascriptInterface
        public void enter() {
            ui.post(() -> sendDownUpKeyEvents(KeyEvent.KEYCODE_ENTER));
        }

        /** Rust を呼んで、結果を __reply(id, json) で返す。 */
        @JavascriptInterface
        public void api(final int id, final String path, final String body) {
            ui.post(() -> {
                InputConnection ic = getCurrentInputConnection();
                CharSequence b = ic == null ? null : ic.getTextBeforeCursor(40, 0);
                final String before = b == null ? "" : b.toString();
                net.execute(() -> {
                    final String res = Native.api(path, body, before);
                    ui.post(() -> {
                        if (web != null) web.evaluateJavascript("window.__reply(" + id + "," + quote(res) + ")", null);
                    });
                });
            });
        }
    }

    private static String quote(String s) {
        return org.json.JSONObject.quote(s == null ? "null" : s);
    }

    @Override
    public void onStartInputView(EditorInfo info, boolean restarting) {
        super.onStartInputView(info, restarting);
        final String pkg = info != null && info.packageName != null ? info.packageName : "";
        int var = info == null ? 0 : (info.inputType & InputType.TYPE_MASK_VARIATION);
        int cls = info == null ? 0 : (info.inputType & InputType.TYPE_MASK_CLASS);
        final boolean secret = var == InputType.TYPE_TEXT_VARIATION_PASSWORD
                || var == InputType.TYPE_TEXT_VARIATION_WEB_PASSWORD
                || var == InputType.TYPE_TEXT_VARIATION_VISIBLE_PASSWORD
                || (cls == InputType.TYPE_CLASS_NUMBER && var == InputType.TYPE_NUMBER_VARIATION_PASSWORD);
        net.execute(() -> Native.setField(pkg, secret));
        if (web != null) web.evaluateJavascript("window.__flush&&window.__flush()", null);
    }

    @Override
    public void onFinishInputView(boolean finishingInput) {
        super.onFinishInputView(finishingInput);
        if (web != null) web.evaluateJavascript("window.__flush&&window.__flush()", null);
    }

    @Override
    public void onDestroy() {
        net.shutdownNow();
        if (web != null) web.destroy();
        super.onDestroy();
    }
}
