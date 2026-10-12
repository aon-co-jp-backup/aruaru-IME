package jp.aon.aruaruime;

import android.graphics.Color;
import android.graphics.Typeface;
import android.inputmethodservice.InputMethodService;
import android.os.Handler;
import android.os.Looper;
import android.view.Gravity;
import android.view.KeyEvent;
import android.view.View;
import android.view.inputmethod.InputConnection;
import android.widget.Button;
import android.widget.HorizontalScrollView;
import android.widget.LinearLayout;
import android.widget.TextView;

import org.json.JSONArray;
import org.json.JSONObject;

import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.UUID;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;

/**
 * aruaru-IME のキーボード(ローマ字入力 → 漢字かな交じりに変換)。変換はサーバー(easy-web.tokyo)のエンジンで行う。
 * 入力はサーバーに送られるが、保存・学習はしない。「あ/A」で、変換しない直接入力に切り替える。
 */
public class AruaruImeService extends InputMethodService {
    private static final String API = "https://easy-web.tokyo/ime/api";
    private final String sid = "and-" + UUID.randomUUID().toString().substring(0, 12);
    private final ExecutorService net = Executors.newSingleThreadExecutor();
    private final Handler ui = new Handler(Looper.getMainLooper());
    private LinearLayout candBar;
    private TextView status;
    private boolean direct = false;   // true: 変換せずそのまま入力
    private boolean shift = false;
    private boolean composing = false;
    private final java.util.ArrayList<Button> letterButtons = new java.util.ArrayList<>();

    @Override
    public View onCreateInputView() {
        final int bg = Color.parseColor("#1b1d22");
        LinearLayout root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setBackgroundColor(bg);
        root.setPadding(dp(2), dp(2), dp(2), dp(4));

        HorizontalScrollView sv = new HorizontalScrollView(this);
        sv.setHorizontalScrollBarEnabled(false);
        candBar = new LinearLayout(this);
        candBar.setOrientation(LinearLayout.HORIZONTAL);
        candBar.setMinimumHeight(dp(48));
        candBar.setGravity(Gravity.CENTER_VERTICAL);
        sv.addView(candBar);
        root.addView(sv, new LinearLayout.LayoutParams(-1, dp(52)));

        String[][] rows = {
            {"1", "2", "3", "4", "5", "6", "7", "8", "9", "0"},
            {"q", "w", "e", "r", "t", "y", "u", "i", "o", "p"},
            {"a", "s", "d", "f", "g", "h", "j", "k", "l", "-"},
            {"z", "x", "c", "v", "b", "n", "m", ",", ".", "?"},
        };
        for (String[] row : rows) {
            LinearLayout r = new LinearLayout(this);
            r.setOrientation(LinearLayout.HORIZONTAL);
            for (String k : row) {
                Button b = key(k, 1f);
                if (k.length() == 1 && Character.isLetter(k.charAt(0))) letterButtons.add(b);
                r.addView(b);
            }
            root.addView(r, new LinearLayout.LayoutParams(-1, dp(48)));
        }
        LinearLayout last = new LinearLayout(this);
        last.setOrientation(LinearLayout.HORIZONTAL);
        final Button mode = action("あ", 1.3f, v -> {
            direct = !direct;
            ((Button) v).setText(direct ? "A" : "あ");
            flushComposing();
            setStatus();
        });
        last.addView(mode);
        last.addView(action("⇧", 1.1f, v -> {
            shift = !shift;
            for (Button b : letterButtons) {
                b.setText(shift ? b.getText().toString().toUpperCase() : b.getText().toString().toLowerCase());
            }
        }));
        last.addView(key("/", 1f));
        last.addView(key("+", 1f));
        last.addView(action("空白 / 変換", 3f, v -> typeKey("Space")));
        last.addView(action("⌫", 1.3f, v -> backspace()));
        last.addView(action("⏎", 1.3f, v -> enter()));
        root.addView(last, new LinearLayout.LayoutParams(-1, dp(52)));

        status = new TextView(this);
        status.setTextColor(Color.parseColor("#888888"));
        status.setTextSize(11);
        status.setGravity(Gravity.CENTER);
        root.addView(status, new LinearLayout.LayoutParams(-1, -2));
        setStatus();
        return root;
    }

    private void setStatus() {
        if (status != null) {
            status.setText(direct ? "aruaru-IME  A: 直接入力 / direct" : "aruaru-IME  あ: ローマ字→日本語(サーバーで変換) / converts on the server");
        }
    }

    private int dp(int v) {
        return (int) (v * getResources().getDisplayMetrics().density + 0.5f);
    }

    private Button key(final String label, float weight) {
        Button b = new Button(this);
        b.setText(label);
        b.setAllCaps(false);
        b.setTextColor(Color.WHITE);
        b.setTextSize(18);
        b.setPadding(0, 0, 0, 0);
        b.setBackgroundColor(Color.parseColor("#2c2f36"));
        LinearLayout.LayoutParams lp = new LinearLayout.LayoutParams(0, -1, weight);
        lp.setMargins(dp(1), dp(1), dp(1), dp(1));
        b.setLayoutParams(lp);
        b.setOnClickListener(v -> typeChar(((Button) v).getText().toString()));
        return b;
    }

    private Button action(String label, float weight, View.OnClickListener l) {
        Button b = key(label, weight);
        b.setBackgroundColor(Color.parseColor("#3b4150"));
        b.setOnClickListener(l);
        return b;
    }

    // ---- 入力の処理 -------------------------------------------------

    private void typeChar(String s) {
        InputConnection ic = getCurrentInputConnection();
        if (ic == null) return;
        if (direct) {
            ic.commitText(s, 1);
            return;
        }
        // 数字は、組み立て中でなければそのまま入力
        if (!composing && s.length() == 1 && Character.isDigit(s.charAt(0))) {
            ic.commitText(s, 1);
            return;
        }
        typeKey("{\"Char\":" + JSONObject.quote(s) + "}");
        if (shift) {
            shift = false;
            for (Button b : letterButtons) b.setText(b.getText().toString().toLowerCase());
        }
    }

    private void typeKey(String keyJson) {
        final InputConnection ic = getCurrentInputConnection();
        if (ic == null) return;
        if (direct) {
            if (keyJson.equals("Space") || keyJson.equals("\"Space\"")) ic.commitText(" ", 1);
            return;
        }
        CharSequence before = ic.getTextBeforeCursor(40, 0);
        final String body = "{\"sid\":\"" + sid + "\",\"key\":" + (keyJson.startsWith("{") ? keyJson : "\"" + keyJson + "\"")
                + ",\"before\":" + JSONObject.quote(before == null ? "" : before.toString()) + "}";
        net.execute(() -> {
            final String res = post("/key", body);
            ui.post(() -> apply(res, keyJson));
        });
    }

    private void backspace() {
        InputConnection ic = getCurrentInputConnection();
        if (ic == null) return;
        if (!direct && composing) {
            typeKey("Backspace");
        } else {
            sendDownUpKeyEvents(KeyEvent.KEYCODE_DEL);
        }
    }

    private void enter() {
        if (!direct && composing) {
            typeKey("Enter");
        } else {
            sendDownUpKeyEvents(KeyEvent.KEYCODE_ENTER);
        }
    }

    private void flushComposing() {
        InputConnection ic = getCurrentInputConnection();
        if (ic != null) ic.finishComposingText();
        composing = false;
        showCandidates(new JSONArray());
        final String body = "{\"sid\":\"" + sid + "\",\"op\":\"cancel\"}";
        net.execute(() -> post("/session", body));
    }

    /** サーバーの応答を、入力欄と候補に反映する。 */
    private void apply(String res, String keyJson) {
        InputConnection ic = getCurrentInputConnection();
        if (ic == null) return;
        try {
            if (res == null) {
                // 通信できないときは、打った文字をそのまま入力する
                if (keyJson.startsWith("{")) ic.commitText(jpPunct(new JSONObject(keyJson).getString("Char")), 1);
                return;
            }
            JSONObject r = new JSONObject(res);
            if (!r.optBoolean("eaten") && r.isNull("commit")) {
                // エンジンが処理しない文字(数字・記号など): そのまま入力
                if (keyJson.startsWith("{")) ic.commitText(new JSONObject(keyJson).getString("Char"), 1);
                else if (keyJson.equals("Space")) ic.commitText(" ", 1);
                return;
            }
            if (!r.isNull("commit")) {
                ic.commitText(r.getString("commit"), 1);
            }
            composing = r.optBoolean("composing");
            String pre = r.optString("preedit", "");
            if (composing && pre.length() > 0) {
                ic.setComposingText(pre, 1);
            } else {
                ic.finishComposingText();
            }
            showCandidates(composing ? r.optJSONArray("cands") : new JSONArray());
        } catch (Exception e) {
            // 解釈できない応答は無視する
        }
    }

    /** 通信できないときの、日本語の句読点(, . → 、。)。 */
    private String jpPunct(String c) {
        if (!direct && c.equals(",")) return "、";
        if (!direct && c.equals(".")) return "。";
        return c;
    }

    private void showCandidates(JSONArray c) {
        if (candBar == null) return;
        candBar.removeAllViews();
        if (c == null) return;
        for (int i = 0; i < c.length() && i < 12; i++) {
            final int idx = i;
            TextView t = new TextView(this);
            t.setText(c.optString(i));
            t.setTextColor(Color.WHITE);
            t.setTextSize(20);
            t.setTypeface(Typeface.DEFAULT);
            t.setPadding(dp(14), dp(8), dp(14), dp(8));
            t.setOnClickListener(v -> choose(idx));
            candBar.addView(t);
        }
    }

    private void choose(final int idx) {
        final String body = "{\"sid\":\"" + sid + "\",\"op\":\"choose\",\"idx\":" + idx + "}";
        net.execute(() -> {
            final String res = post("/session", body);
            ui.post(() -> apply(res, "choose"));
        });
    }

    // ---- 通信 --------------------------------------------------------

    private String post(String path, String body) {
        HttpURLConnection c = null;
        try {
            c = (HttpURLConnection) new URL(API + path).openConnection();
            c.setRequestMethod("POST");
            c.setConnectTimeout(4000);
            c.setReadTimeout(6000);
            c.setDoOutput(true);
            c.setRequestProperty("Content-Type", "application/json");
            byte[] b = body.getBytes("UTF-8");
            try (OutputStream o = c.getOutputStream()) {
                o.write(b);
            }
            if (c.getResponseCode() != 200) return null;
            java.io.InputStream in = c.getInputStream();
            java.io.ByteArrayOutputStream out = new java.io.ByteArrayOutputStream();
            byte[] buf = new byte[4096];
            int n;
            while ((n = in.read(buf)) > 0) out.write(buf, 0, n);
            return out.toString("UTF-8");
        } catch (Exception e) {
            return null;
        } finally {
            if (c != null) c.disconnect();
        }
    }

    @Override
    public void onStartInputView(android.view.inputmethod.EditorInfo info, boolean restarting) {
        super.onStartInputView(info, restarting);
        composing = false;
        showCandidates(new JSONArray());
    }

    @Override
    public void onFinishInputView(boolean finishingInput) {
        super.onFinishInputView(finishingInput);
        flushComposing();
    }

    @Override
    public void onDestroy() {
        net.shutdownNow();
        super.onDestroy();
    }
}
