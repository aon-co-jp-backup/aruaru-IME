package jp.aon.aruaruime;

import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.content.pm.PackageInstaller;
import android.os.Build;

import org.json.JSONArray;
import org.json.JSONObject;

import java.io.File;
import java.io.FileInputStream;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.io.OutputStream;
import java.net.HttpURLConnection;
import java.net.URL;
import java.security.MessageDigest;

/**
 * 自動アップデート(Android)。GitHub Releases の最新 APK を確認し、取得して(SHA-256 があれば検証)、
 * 利用者の確認つきでインストールする。Android は、通常のアプリが無確認で自分を入れ替えることを許さないため、
 * 最後の「更新する」の確認だけは OS の画面になる。署名鍵が同じなので、データは残ったまま上書きされる。
 * 確認と取得は、アプリの起動時とキーボードの初回表示時に、6時間に1回まで行う。
 */
public final class Updater {
    static final String REPO = "aon-co-jp/aruaru-IME";
    static final String APK_NAME = "aruaru-ime-android.apk";
    private static final String PREFS = "aruaru_update";
    private static final long EVERY_MS = 6L * 3600 * 1000;
    public static final String ACTION_STATUS = "jp.aon.aruaruime.INSTALL_STATUS";

    private Updater() {}

    /** 取得済みで、まだ入れていない新しい版。 */
    public static final class Pending {
        public final String tag;
        public final File apk;
        Pending(String tag, File apk) { this.tag = tag; this.apk = apk; }
    }

    static int[] parse(String s) {
        String core = s.trim().replaceFirst("^[vV]", "").split("[-+]")[0];
        String[] p = core.split("\\.");
        int[] v = new int[3];
        try {
            for (int i = 0; i < 3 && i < p.length; i++) v[i] = Integer.parseInt(p[i]);
        } catch (NumberFormatException e) {
            return null;
        }
        return v;
    }

    static int cmp(int[] a, int[] b) {
        for (int i = 0; i < 3; i++) if (a[i] != b[i]) return Integer.compare(a[i], b[i]);
        return 0;
    }

    static boolean trusted(String u) {
        if (u == null || !u.startsWith("https://")) return false;
        String host = u.substring(8).split("/")[0];
        return host.equals("github.com") || host.endsWith(".githubusercontent.com");
    }

    static String installedVersion(Context c) {
        try {
            return c.getPackageManager().getPackageInfo(c.getPackageName(), 0).versionName;
        } catch (Exception e) {
            return "0.0.0";
        }
    }

    private static byte[] readAll(String url, int max) throws Exception {
        HttpURLConnection h = (HttpURLConnection) new URL(url).openConnection();
        h.setConnectTimeout(15000);
        h.setReadTimeout(60000);
        h.setRequestProperty("User-Agent", "aruaru-ime-updater");
        try (InputStream in = h.getInputStream()) {
            java.io.ByteArrayOutputStream b = new java.io.ByteArrayOutputStream();
            byte[] buf = new byte[16384];
            int n;
            while ((n = in.read(buf)) > 0) {
                b.write(buf, 0, n);
                if (b.size() > max) throw new java.io.IOException("too large");
            }
            return b.toByteArray();
        }
    }

    /** 6時間に1回まで、新しい版を確認して取得する(ネットワーク使用のため、必ずバックグラウンドで呼ぶ)。 */
    public static void maintenance(Context ctx, boolean force) {
        SharedPreferences sp = ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE);
        long now = System.currentTimeMillis();
        if (!force && now - sp.getLong("checked", 0) < EVERY_MS) return;
        sp.edit().putLong("checked", now).apply();
        try {
            int[] cur = parse(installedVersion(ctx));
            JSONArray list = new JSONArray(new String(readAll("https://api.github.com/repos/" + REPO + "/releases?per_page=15", 2000000), "UTF-8"));
            int[] best = null;
            String bestTag = null, apkUrl = null, shaUrl = null;
            for (int i = 0; i < list.length(); i++) {
                JSONObject r = list.getJSONObject(i);
                if (r.optBoolean("draft")) continue;
                String tag = r.optString("tag_name");
                int[] v = parse(tag);
                if (v == null || cmp(v, cur) <= 0 || (best != null && cmp(v, best) <= 0)) continue;
                JSONArray assets = r.optJSONArray("assets");
                String a = null, s = null;
                for (int j = 0; assets != null && j < assets.length(); j++) {
                    JSONObject as = assets.getJSONObject(j);
                    String n = as.optString("name");
                    if (n.equals(APK_NAME)) a = as.optString("browser_download_url");
                    if (n.equals(APK_NAME + ".sha256")) s = as.optString("browser_download_url");
                }
                if (a != null && trusted(a) && (s == null || trusted(s))) {
                    best = v; bestTag = tag; apkUrl = a; shaUrl = s;
                }
            }
            if (best == null) return;
            File dir = new File(ctx.getCacheDir(), "update");
            dir.mkdirs();
            File out = new File(dir, "aruaru-ime-" + bestTag.replaceAll("[^0-9A-Za-z.]", "_") + ".apk");
            if (!out.isFile()) {
                byte[] bytes = readAll(apkUrl, 200 * 1024 * 1024);
                if (shaUrl != null) {
                    String want = new String(readAll(shaUrl, 4096), "UTF-8").trim().split("\\s+")[0].toLowerCase();
                    StringBuilder got = new StringBuilder();
                    for (byte x : MessageDigest.getInstance("SHA-256").digest(bytes)) got.append(String.format("%02x", x));
                    if (!want.equals(got.toString())) return; // ハッシュ不一致: 入れない
                }
                File tmp = new File(dir, out.getName() + ".part");
                try (OutputStream o = new FileOutputStream(tmp)) { o.write(bytes); }
                tmp.renameTo(out);
            }
            sp.edit().putString("pending_tag", bestTag).putString("pending_apk", out.getAbsolutePath()).apply();
        } catch (Exception ignored) {
            // 通信できない等は黙って次回へ
        }
    }

    /** 取得済みで、いま入っている版より新しい APK があれば返す。 */
    public static Pending pending(Context ctx) {
        SharedPreferences sp = ctx.getSharedPreferences(PREFS, Context.MODE_PRIVATE);
        String tag = sp.getString("pending_tag", null), path = sp.getString("pending_apk", null);
        if (tag == null || path == null) return null;
        int[] v = parse(tag), cur = parse(installedVersion(ctx));
        File f = new File(path);
        if (v == null || cur == null || cmp(v, cur) <= 0 || !f.isFile()) {
            if (f.isFile() && v != null && cur != null && cmp(v, cur) <= 0) f.delete();
            sp.edit().remove("pending_tag").remove("pending_apk").apply();
            return null;
        }
        return new Pending(tag, f);
    }

    /** 「提供元不明のアプリ」の許可がまだなら false(設定画面へ案内する)。 */
    public static boolean canInstall(Context ctx) {
        return Build.VERSION.SDK_INT < 26 || ctx.getPackageManager().canRequestPackageInstalls();
    }

    /** PackageInstaller で入れる。最後の確認画面は OS が出す(結果は ACTION_STATUS のブロードキャストで届く)。 */
    public static void install(Context ctx, Pending p) throws Exception {
        PackageInstaller pi = ctx.getPackageManager().getPackageInstaller();
        PackageInstaller.SessionParams params = new PackageInstaller.SessionParams(PackageInstaller.SessionParams.MODE_FULL_INSTALL);
        int id = pi.createSession(params);
        try (PackageInstaller.Session s = pi.openSession(id)) {
            try (InputStream in = new FileInputStream(p.apk); OutputStream out = s.openWrite("aruaru-ime.apk", 0, p.apk.length())) {
                byte[] buf = new byte[65536];
                int n;
                while ((n = in.read(buf)) > 0) out.write(buf, 0, n);
                s.fsync(out);
            }
            Intent i = new Intent(ACTION_STATUS).setPackage(ctx.getPackageName());
            int flags = PendingIntent.FLAG_UPDATE_CURRENT | (Build.VERSION.SDK_INT >= 31 ? PendingIntent.FLAG_MUTABLE : 0);
            s.commit(PendingIntent.getBroadcast(ctx, id, i, flags).getIntentSender());
        }
    }
}
