package jp.aon.aruaruime;

import android.app.Activity;
import android.app.AlertDialog;
import android.content.Context;
import android.content.Intent;
import android.provider.Settings;
import android.view.inputmethod.InputMethodInfo;
import android.view.inputmethod.InputMethodManager;
import android.os.Bundle;
import android.webkit.WebChromeClient;
import android.webkit.WebSettings;
import android.webkit.WebView;
import android.webkit.WebViewClient;

/**
 * aruaru-IME for Android (WebView shell). Opens the online screen https://easy-web.tokyo/ime,
 * which converts on the server (input is sent to the server but not stored or learned).
 * Phones and tablets: portrait and landscape are both handled by the page layout.
 */
public class MainActivity extends Activity {
    private static final String HOME = "https://easy-web.tokyo/ime";
    private WebView web;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        web = new WebView(this);
        setContentView(web);
        WebSettings s = web.getSettings();
        s.setJavaScriptEnabled(true);
        s.setDomStorageEnabled(true);
        s.setMixedContentMode(WebSettings.MIXED_CONTENT_NEVER_ALLOW);
        web.setWebViewClient(new WebViewClient());
        web.setWebChromeClient(new WebChromeClient());
        offerKeyboard();
        if (savedInstanceState != null) {
            web.restoreState(savedInstanceState);
        } else {
            web.loadUrl(HOME);
        }
    }

    /** aruaru-IME のキーボードがまだ有効でなければ、設定を案内する。 */
    private void offerKeyboard() {
        try {
            InputMethodManager imm = (InputMethodManager) getSystemService(Context.INPUT_METHOD_SERVICE);
            boolean enabled = false;
            for (InputMethodInfo i : imm.getEnabledInputMethodList()) {
                if (i.getPackageName().equals(getPackageName())) enabled = true;
            }
            if (enabled) return;
            new AlertDialog.Builder(this)
                .setTitle("aruaru-IME キーボード / keyboard")
                .setMessage("どのアプリでも日本語入力できるキーボードを有効にします。「設定を開く」で aruaru-IME をオンにし、入力欄で aruaru-IME を選んでください。\nEnable the aruaru-IME keyboard to type Japanese in any app.")
                .setPositiveButton("設定を開く / Open settings", (d, w) -> startActivity(new Intent(Settings.ACTION_INPUT_METHOD_SETTINGS)))
                .setNegativeButton("あとで / Later", null)
                .show();
        } catch (Exception ignored) {
        }
    }

    @Override
    protected void onSaveInstanceState(Bundle out) {
        super.onSaveInstanceState(out);
        web.saveState(out);
    }

    @Override
    public void onBackPressed() {
        if (web.canGoBack()) {
            web.goBack();
        } else {
            super.onBackPressed();
        }
    }
}
