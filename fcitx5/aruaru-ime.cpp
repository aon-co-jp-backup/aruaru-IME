// aruaru-IME の Fcitx5 アドオン: キーを常駐エンジン(127.0.0.1:4660)へ送り、変換結果を Fcitx5 へ反映する。
#include <fcitx/addonfactory.h>
#include <fcitx/addonmanager.h>
#include <fcitx/candidatelist.h>
#include <fcitx/inputcontext.h>
#include <fcitx/inputmethodengine.h>
#include <fcitx/inputpanel.h>
#include <fcitx/instance.h>
#include <fcitx/text.h>
#include <fcitx-utils/key.h>
#include <arpa/inet.h>
#include <sys/socket.h>
#include <sys/time.h>
#include <unistd.h>
#include <cstdint>
#include <cstdlib>
#include <map>
#include <memory>
#include <string>
#include <vector>

namespace {

// ---- 最小の JSON(エンジンの応答だけを読む) ----
struct J {
    enum T { Null, Bool, Num, Str, Arr, Obj } t = Null;
    bool b = false;
    double n = 0;
    std::string s;
    std::vector<J> a;
    std::map<std::string, J> o;
    const J &operator[](const std::string &k) const {
        static J nul;
        auto i = o.find(k);
        return i == o.end() ? nul : i->second;
    }
};

struct P {
    const std::string &x;
    size_t i = 0;
    void ws() { while (i < x.size() && (x[i] == ' ' || x[i] == '\n' || x[i] == '\r' || x[i] == '\t')) i++; }
    static void utf8(std::string &o, uint32_t c) {
        if (c < 0x80) o += char(c);
        else if (c < 0x800) { o += char(0xC0 | (c >> 6)); o += char(0x80 | (c & 63)); }
        else if (c < 0x10000) { o += char(0xE0 | (c >> 12)); o += char(0x80 | ((c >> 6) & 63)); o += char(0x80 | (c & 63)); }
        else { o += char(0xF0 | (c >> 18)); o += char(0x80 | ((c >> 12) & 63)); o += char(0x80 | ((c >> 6) & 63)); o += char(0x80 | (c & 63)); }
    }
    uint32_t hex4() {
        uint32_t v = std::strtoul(x.substr(i, 4).c_str(), nullptr, 16);
        i += 4;
        return v;
    }
    std::string str() {
        std::string o;
        i++;  // 開きの引用符
        while (i < x.size() && x[i] != '"') {
            if (x[i] == '\\' && i + 1 < x.size()) {
                char c = x[++i];
                i++;
                switch (c) {
                case 'n': o += '\n'; break;
                case 't': o += '\t'; break;
                case 'r': o += '\r'; break;
                case 'b': o += '\b'; break;
                case 'f': o += '\f'; break;
                case 'u': {
                    uint32_t u = hex4();
                    if (u >= 0xD800 && u < 0xDC00 && x.compare(i, 2, "\\u") == 0) {
                        i += 2;
                        uint32_t l = hex4();
                        u = 0x10000 + ((u - 0xD800) << 10) + (l - 0xDC00);
                    }
                    utf8(o, u);
                    break;
                }
                default: o += c;
                }
            } else {
                o += x[i++];
            }
        }
        i++;
        return o;
    }
    J val() {
        J v;
        ws();
        if (i >= x.size()) return v;
        char c = x[i];
        if (c == '{') {
            v.t = J::Obj;
            i++;
            ws();
            while (i < x.size() && x[i] != '}') {
                ws();
                std::string k = str();
                ws();
                i++;  // コロン
                v.o[k] = val();
                ws();
                if (i < x.size() && x[i] == ',') i++;
                ws();
            }
            i++;
        } else if (c == '[') {
            v.t = J::Arr;
            i++;
            ws();
            while (i < x.size() && x[i] != ']') {
                v.a.push_back(val());
                ws();
                if (i < x.size() && x[i] == ',') i++;
                ws();
            }
            i++;
        } else if (c == '"') {
            v.t = J::Str;
            v.s = str();
        } else if (x.compare(i, 4, "true") == 0) { v.t = J::Bool; v.b = true; i += 4; }
        else if (x.compare(i, 5, "false") == 0) { v.t = J::Bool; i += 5; }
        else if (x.compare(i, 4, "null") == 0) { i += 4; }
        else {
            v.t = J::Num;
            size_t st = i;
            while (i < x.size() && (isdigit((unsigned char)x[i]) || x[i] == '-' || x[i] == '.' || x[i] == 'e' || x[i] == 'E' || x[i] == '+')) i++;
            v.n = std::atof(x.substr(st, i - st).c_str());
        }
        return v;
    }
};

int enginePort() {
    const char *p = std::getenv("ARUARU_IME_PORT");
    return p && *p ? std::atoi(p) : 4660;
}

// HTTP を直接話す(接続を閉じるまで読む)。失敗したら空文字。
std::string http(const std::string &method, const std::string &path, const std::string &body) {
    int fd = socket(AF_INET, SOCK_STREAM, 0);
    if (fd < 0) return "";
    timeval tv{1, 500000};
    setsockopt(fd, SOL_SOCKET, SO_RCVTIMEO, &tv, sizeof tv);
    setsockopt(fd, SOL_SOCKET, SO_SNDTIMEO, &tv, sizeof tv);
    sockaddr_in a{};
    a.sin_family = AF_INET;
    a.sin_port = htons(enginePort());
    inet_pton(AF_INET, "127.0.0.1", &a.sin_addr);
    if (connect(fd, (sockaddr *)&a, sizeof a) != 0) { close(fd); return ""; }
    std::string req = method + " " + path + " HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\nContent-Type: application/json\r\nContent-Length: " + std::to_string(body.size()) + "\r\n\r\n" + body;
    size_t off = 0;
    while (off < req.size()) {
        ssize_t n = send(fd, req.data() + off, req.size() - off, MSG_NOSIGNAL);
        if (n <= 0) { close(fd); return ""; }
        off += n;
    }
    std::string resp;
    char buf[8192];
    ssize_t n;
    while ((n = recv(fd, buf, sizeof buf, 0)) > 0) resp.append(buf, n);
    close(fd);
    auto h = resp.find("\r\n\r\n");
    return h == std::string::npos ? "" : resp.substr(h + 4);
}

std::string keyJson(const fcitx::Key &key) {
    auto st = key.states();
    if (st.test(fcitx::KeyState::Ctrl) || st.test(fcitx::KeyState::Alt) || st.test(fcitx::KeyState::Super)) return "";
    uint32_t sym = static_cast<uint32_t>(key.sym());
    switch (sym) {
    case 0x20: return "\"Space\"";
    case 0xff0d: return "\"Enter\"";
    case 0xff1b: return "\"Esc\"";
    case 0xff08: return "\"Backspace\"";
    case 0xff52: return "\"Up\"";
    case 0xff54: return "\"Down\"";
    case 0xff51: return "\"Left\"";
    case 0xff53: return "\"Right\"";
    case 0xff09: return "\"Tab\"";
    case 0xff23: return "\"Convert\"";
    case 0xff22: return "\"NonConvert\"";
    case 0xff2a: return "\"Toggle\"";
    }
    if (sym >= 0xffc3 && sym <= 0xffc7) return "{\"Fn\":" + std::to_string(sym - 0xffc3 + 6) + "}";
    if (sym >= 0x21 && sym <= 0x7e) {
        std::string c(1, char(sym));
        if (c == "\"" || c == "\\") c = "\\" + c;
        return "{\"Char\":\"" + c + "\"}";
    }
    return "";
}

std::string sid(fcitx::InputContext *ic) { return "fcitx5-" + std::to_string(reinterpret_cast<uintptr_t>(ic)); }

}  // namespace

class AruaruEngine : public fcitx::InputMethodEngineV2 {
public:
    explicit AruaruEngine(fcitx::Instance *) {
        if (http("GET", "/health", "").empty()) {
            // 常駐エンジンが居なければ起動を試みる(PATH の aruaru-ime-daemon)
            if (std::system("(aruaru-ime-daemon >/dev/null 2>&1 &)") != 0) {}
            for (int i = 0; i < 20 && http("GET", "/health", "").empty(); i++) usleep(250000);
        }
    }

    void keyEvent(const fcitx::InputMethodEntry &, fcitx::KeyEvent &ev) override {
        if (ev.isRelease()) return;
        std::string k = keyJson(ev.key());
        if (k.empty()) return;
        auto *ic = ev.inputContext();
        std::string text = http("POST", "/key", "{\"sid\":\"" + sid(ic) + "\",\"key\":" + k + "}");
        if (text.empty()) return;
        P p{text};
        J r = p.val();
        bool eaten = r["eaten"].b;
        bool hasCommit = !r["commit"].s.empty();
        if (!eaten && !hasCommit) return;
        ev.filterAndAccept();
        apply(ic, r);
    }

    void reset(const fcitx::InputMethodEntry &, fcitx::InputContextEvent &ev) override {
        auto *ic = ev.inputContext();
        http("POST", "/session", "{\"sid\":\"" + sid(ic) + "\",\"op\":\"cancel\"}");
        ic->inputPanel().reset();
        ic->updatePreedit();
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    }

private:
    void apply(fcitx::InputContext *ic, const J &r) {
        if (!r["commit"].s.empty()) ic->commitString(r["commit"].s);
        auto &panel = ic->inputPanel();
        panel.reset();
        const std::string &pre = r["preedit"].s;
        if (r["composing"].b && !pre.empty()) {
            fcitx::Text t(pre, fcitx::TextFormatFlag::Underline);
            t.setCursor(static_cast<int>(pre.size()));
            if (ic->capabilityFlags().test(fcitx::CapabilityFlag::Preedit)) panel.setClientPreedit(t);
            else panel.setPreedit(t);
            if (!r["cands"].a.empty()) {
                auto list = std::make_unique<fcitx::CommonCandidateList>();
                for (auto &c : r["cands"].a) list->append<fcitx::DisplayOnlyCandidateWord>(fcitx::Text(c.s));
                int sel = static_cast<int>(r["sel"].n);
                if (sel >= 0 && sel < list->size()) list->setGlobalCursorIndex(sel);
                panel.setCandidateList(std::move(list));
            }
        }
        ic->updatePreedit();
        ic->updateUserInterface(fcitx::UserInterfaceComponent::InputPanel);
    }
};

class AruaruFactory : public fcitx::AddonFactory {
public:
    fcitx::AddonInstance *create(fcitx::AddonManager *m) override { return new AruaruEngine(m->instance()); }
};

FCITX_ADDON_FACTORY(AruaruFactory)
