// aruaru-IME for macOS (InputMethodKit). Talks to the resident engine (aruaru-ime-daemon, 127.0.0.1:4660) over HTTP.
// Keys go to /key; the reply's commit / preedit / candidates are shown with the system's marked text and candidate panel.
import Cocoa
import InputMethodKit

let kConnectionName = "jp.aon.aruaru-IME_Connection"
var imkServer: IMKServer! = nil

let enginePort = ProcessInfo.processInfo.environment["ARUARU_IME_PORT"] ?? "4660"

/// Synchronous JSON POST to the engine. Returns nil when the engine is not reachable.
func post(_ path: String, _ body: [String: Any]) -> [String: Any]? {
    guard let url = URL(string: "http://127.0.0.1:\(enginePort)\(path)"),
          let data = try? JSONSerialization.data(withJSONObject: body) else { return nil }
    var req = URLRequest(url: url, timeoutInterval: 1.5)
    req.httpMethod = "POST"
    req.httpBody = data
    var result: [String: Any]? = nil
    let sem = DispatchSemaphore(value: 0)
    URLSession.shared.dataTask(with: req) { d, _, _ in
        if let d = d, let j = try? JSONSerialization.jsonObject(with: d) as? [String: Any] { result = j }
        sem.signal()
    }.resume()
    _ = sem.wait(timeout: .now() + 2)
    return result
}

/// Start the engine if it is not running (installed next to the app by install.sh).
func ensureEngine() {
    if post("/session", ["sid": "probe", "op": "cancel"]) != nil { return }
    let candidates = [NSHomeDirectory() + "/.local/share/aruaru-ime/bin/aruaru-ime-daemon"]
    for c in candidates where FileManager.default.isExecutableFile(atPath: c) {
        let p = Process()
        p.executableURL = URL(fileURLWithPath: c)
        try? p.run()
        for _ in 0..<20 {
            Thread.sleep(forTimeInterval: 0.25)
            if post("/session", ["sid": "probe", "op": "cancel"]) != nil { return }
        }
    }
}

@objc(AruaruIMEController)
class AruaruIMEController: IMKInputController {
    let sid = "mac-" + UUID().uuidString.prefix(8)
    var cands: [String] = []
    var composing = false
    lazy var panel: IMKCandidates = IMKCandidates(server: imkServer, panelType: kIMKSingleColumnScrollingCandidatePanel)

    /// macOS key code / characters → the engine's Key JSON ("Space" or {"Char":"a"}).
    func keyJSON(_ event: NSEvent) -> Any? {
        if !event.modifierFlags.intersection([.command, .control, .option]).isEmpty { return nil }
        switch event.keyCode {
        case 36, 76: return "Enter"
        case 49: return "Space"
        case 51: return "Backspace"
        case 53: return "Esc"
        case 126: return "Up"
        case 125: return "Down"
        case 123: return "Left"
        case 124: return "Right"
        case 48: return "Tab"
        case 102: return "Off"   // eisu
        case 104: return "On"    // kana
        default: break
        }
        if let ch = event.characters, ch.unicodeScalars.count == 1, let u = ch.unicodeScalars.first, u.value >= 0x21, u.value <= 0x7e {
            return ["Char": ch]
        }
        return nil
    }

    func apply(_ r: [String: Any], _ client: IMKTextInput) {
        let range = NSRange(location: NSNotFound, length: NSNotFound)
        if let c = r["commit"] as? String, !c.isEmpty {
            client.insertText(c, replacementRange: range)
        }
        composing = (r["composing"] as? Bool) ?? false
        let pre = (r["preedit"] as? String) ?? ""
        if composing && !pre.isEmpty {
            client.setMarkedText(pre, selectionRange: NSRange(location: (pre as NSString).length, length: 0), replacementRange: range)
            cands = (r["cands"] as? [String]) ?? []
            if cands.isEmpty { panel.hide() } else { panel.update(); panel.show() }
        } else {
            client.setMarkedText("", selectionRange: NSRange(location: 0, length: 0), replacementRange: range)
            cands = []
            panel.hide()
        }
    }

    override func handle(_ event: NSEvent!, client sender: Any!) -> Bool {
        guard let event = event, event.type == .keyDown, let client = sender as? IMKTextInput,
              let key = keyJSON(event) else { return false }
        var before = ""
        let sel = client.selectedRange()
        if sel.location != NSNotFound && sel.location > 0 {
            let from = max(0, sel.location - 40)
            before = client.attributedSubstring(from: NSRange(location: from, length: sel.location - from))?.string ?? ""
        }
        guard let r = post("/key", ["sid": sid, "key": key, "before": before]) else { return false }
        let eaten = (r["eaten"] as? Bool) ?? false
        let hasCommit = ((r["commit"] as? String) ?? "").isEmpty == false
        if !eaten && !hasCommit { return false }
        apply(r, client)
        return true
    }

    override func candidates(_ sender: Any!) -> [Any]! { return cands }

    override func candidateSelected(_ candidateString: NSAttributedString!) {
        guard let s = candidateString?.string, let idx = cands.firstIndex(of: s), let client = self.client() else { return }
        if let r = post("/session", ["sid": sid, "op": "choose", "idx": idx]) { apply(r, client) }
    }

    override func deactivateServer(_ sender: Any!) {
        _ = post("/session", ["sid": sid, "op": "cancel"])
        panel.hide()
        super.deactivateServer(sender)
    }
}

autoreleasepool {
    ensureEngine()
    imkServer = IMKServer(name: kConnectionName, bundleIdentifier: Bundle.main.bundleIdentifier)
    NSApplication.shared.run()
}
