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
        // エミュレーター(GPU が仮想)では、WebView が真っ黒になることがあるので、ソフトウェア描画にする(実機はそのまま)
        if (android.os.Build.FINGERPRINT.startsWith("generic") || android.os.Build.FINGERPRINT.contains("emulator") || android.os.Build.MODEL.contains("Emulator") || android.os.Build.MODEL.startsWith("sdk")) {
            web.setLayerType(android.view.View.LAYER_TYPE_SOFTWARE, null);
        }
        WebSettings s = web.getSettings();
        s.setJavaScriptEnabled(true);
        s.setDomStorageEnabled(true);
        s.setMixedContentMode(WebSettings.MIXED_CONTENT_NEVER_ALLOW);
        web.setWebViewClient(new WebViewClient() {
            @Override
            public void onReceivedError(WebView view, android.webkit.WebResourceRequest request, android.webkit.WebResourceError error) {
                // 通信できないとき(本体のページ読み込みの失敗)は、分かりやすい案内と再読み込みのリンクを出す
                if (request.isForMainFrame()) {
                    view.loadDataWithBaseURL(HOME, "<html><head><meta name='viewport' content='width=device-width,initial-scale=1'></head>"
                        + "<body style='font:18px sans-serif;padding:24px;background:#111;color:#fff'>"
                        + "<h3>aruaru-IME</h3><p>インターネットにつながっていないようです。つながってから、もう一度お試しください。<br>No internet connection. Please try again when you are online.</p>"
                        + "<p><a style='color:#4a9eff' href='" + HOME + "'>再読み込み / Retry</a></p></body></html>", "text/html", "UTF-8", null);
                }
            }
        });
        web.setWebChromeClient(new WebChromeClient());
        offerKeyboard();
        startMaintenance();
        if (savedInstanceState != null) {
            web.restoreState(savedInstanceState);
        } else {
            web.loadUrl(HOME);
        }
    }

    private final android.content.BroadcastReceiver installStatus = new android.content.BroadcastReceiver() {
        @Override
        public void onReceive(Context c, Intent i) {
            int st = i.getIntExtra(android.content.pm.PackageInstaller.EXTRA_STATUS, -1);
            if (st == android.content.pm.PackageInstaller.STATUS_PENDING_USER_ACTION) {
                Intent confirm = i.getParcelableExtra(Intent.EXTRA_INTENT);
                if (confirm != null) {
                    confirm.addFlags(Intent.FLAG_ACTIVITY_NEW_TASK);
                    startActivity(confirm); // OS の「更新しますか?」画面
                }
            }
        }
    };

    /** 起動時のメンテナンス: 新しい版を確認・取得し、あれば更新の案内を出す(6時間に1回まで確認)。 */
    private void startMaintenance() {
        try {
            if (android.os.Build.VERSION.SDK_INT >= 33) registerReceiver(installStatus, new android.content.IntentFilter(Updater.ACTION_STATUS), Context.RECEIVER_NOT_EXPORTED);
            else registerReceiver(installStatus, new android.content.IntentFilter(Updater.ACTION_STATUS));
        } catch (Exception ignored) {
        }
        offerUpdate(); // 前回までに取得済みのものがあれば、すぐ案内する
        new Thread(() -> {
            Updater.maintenance(getApplicationContext(), false);
            runOnUiThread(this::offerUpdate);
        }).start();
    }

    private boolean updateDialogShown = false;

    private void offerUpdate() {
        final Updater.Pending p = Updater.pending(this);
        if (p == null || updateDialogShown || isFinishing()) return;
        updateDialogShown = true;
        new AlertDialog.Builder(this)
            .setTitle("新しい版 " + p.tag + " / New version")
            .setMessage("aruaru-IME の新しい版を取得しました。更新しますか?(データは残ります)\nA new version of aruaru-IME is ready. Update now? (your data is kept)")
            .setPositiveButton("更新する / Update", (d, w) -> {
                try {
                    if (!Updater.canInstall(this)) {
                        startActivity(new Intent(Settings.ACTION_MANAGE_UNKNOWN_APP_SOURCES, android.net.Uri.parse("package:" + getPackageName())));
                        updateDialogShown = false; // 許可したあと戻ってきたら、もう一度案内する
                        return;
                    }
                    Updater.install(this, p);
                } catch (Exception e) {
                    updateDialogShown = false;
                }
            })
            .setNegativeButton("あとで / Later", null)
            .show();
    }

    @Override
    protected void onResume() {
        super.onResume();
        offerUpdate();
    }

    @Override
    protected void onDestroy() {
        try { unregisterReceiver(installStatus); } catch (Exception ignored) { }
        super.onDestroy();
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
