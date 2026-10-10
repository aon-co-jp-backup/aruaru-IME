using System;
using System.IO;
using System.Runtime.InteropServices;
using System.Windows.Forms;

// aruaru-IME 動作確認用の使い捨てウィンドウ。
//  - 自分のプロセスだけで aruaru-IME を有効にする(他のアプリの入力方式には影響しない)
//  - テキストボックスの内容を %TEMP%\testhost.txt に書き出す(自動確認用)
class TestHost {
    [ComImport, Guid("71c6e74c-0f28-11d8-a82a-00065b84435c"), InterfaceType(ComInterfaceType.InterfaceIsIUnknown)]
    interface ITfInputProcessorProfileMgr {
        [PreserveSig] int ActivateProfile(uint profileType, ushort langid, ref Guid clsid, ref Guid profile, IntPtr hkl, uint flags);
    }

    static string Activate(bool bypass) {
        try {
            var t = Type.GetTypeFromCLSID(new Guid("33C53A50-F456-4884-B049-85FD643ECFED"));
            var mgr = (ITfInputProcessorProfileMgr)Activator.CreateInstance(t);
            var clsid = new Guid("B7C2A1E4-5D3F-4F7A-9A61-0C8E2D4B6A11");
            var prof = new Guid("C1D3E5F7-2A4B-4C6D-8E90-1F2A3B4C5D6E");
            // TF_PROFILETYPE_INPUTPROCESSOR=1, TF_IPPMF_FORPROCESS=0x10000000 | DONTCARECURRENTINPUTLANGUAGE=4
            int hr = mgr.ActivateProfile(1, 0x0411, ref clsid, ref prof, IntPtr.Zero, 0x10000000 | 4);
            return "ActivateProfile hr=0x" + hr.ToString("X");
        } catch (Exception e) {
            return "Activate failed: " + e.Message;
        }
    }

    [STAThread]
    static void Main(string[] args) {
        var f = new Form { Text = "aruaru-IME TestHost", Width = 700, Height = 400, TopMost = true };
        var t = new TextBox { Multiline = true, Dock = DockStyle.Fill, Font = new System.Drawing.Font("Yu Gothic UI", 16) };
        f.Controls.Add(t);
        var l = new Label { Dock = DockStyle.Bottom, Height = 28, Text = "" };
        f.Controls.Add(l);
        string status = "";
        f.Shown += (s, e) => { t.Focus(); if (args.Length == 0 || args[0] != "--plain") status = Activate(false); };
        var path = Path.Combine(Path.GetTempPath(), "testhost.txt");
        var timer = new Timer { Interval = 200 };
        timer.Tick += (s, e) => {
            l.Text = status + "  len=" + t.Text.Length;
            try { File.WriteAllText(path, status + "\n" + t.Text); } catch { }
        };
        timer.Start();
        Application.Run(f);
    }
}
