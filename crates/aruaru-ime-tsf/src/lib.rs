//! aruaru-IME Windows text service (TSF) DLL。regsvr32 で登録する(管理者権限)。
#![cfg(windows)]

mod candwin;
mod daemon;
mod service;

use std::ffi::c_void;
use std::sync::atomic::{AtomicIsize, AtomicU32, Ordering};
use windows::core::{implement, w, IUnknown, Interface, Result, GUID, HRESULT, PCWSTR};
use windows::Win32::Foundation::*;
use windows::Win32::System::Com::*;
use windows::Win32::System::LibraryLoader::GetModuleFileNameW;
use windows::Win32::System::Registry::*;
use windows::Win32::UI::Input::KeyboardAndMouse::HKL;
use windows::Win32::UI::TextServices::*;

/// テキストサービスの CLSID。
pub const CLSID_ARUARU_IME: GUID = GUID::from_u128(0xB7C2A1E4_5D3F_4F7A_9A61_0C8E2D4B6A11);
/// 言語プロファイル(日本語)の GUID。
pub const GUID_PROFILE: GUID = GUID::from_u128(0xC1D3E5F7_2A4B_4C6D_8E90_1F2A3B4C5D6E);
const LANGID_JA: u16 = 0x0411;
const DESCRIPTION: &str = "aruaru-IME (AI)";

static MODULE: AtomicIsize = AtomicIsize::new(0);
static LOCKS: AtomicU32 = AtomicU32::new(0);

pub fn dll_path() -> std::path::PathBuf {
    let mut buf = [0u16; 520];
    let n = unsafe { GetModuleFileNameW(HMODULE(MODULE.load(Ordering::Relaxed) as *mut c_void), &mut buf) } as usize;
    std::path::PathBuf::from(String::from_utf16_lossy(&buf[..n]))
}

pub fn dll_dir() -> std::path::PathBuf {
    dll_path().parent().map(|p| p.to_path_buf()).unwrap_or_default()
}

#[no_mangle]
extern "system" fn DllMain(hinst: HMODULE, reason: u32, _reserved: *mut c_void) -> BOOL {
    if reason == 1 {
        MODULE.store(hinst.0 as isize, Ordering::Relaxed);
    }
    TRUE
}

#[implement(IClassFactory)]
struct Factory;

impl IClassFactory_Impl for Factory_Impl {
    fn CreateInstance(&self, outer: Option<&IUnknown>, riid: *const GUID, ppv: *mut *mut c_void) -> Result<()> {
        if outer.is_some() {
            return Err(CLASS_E_NOAGGREGATION.into());
        }
        let ts: IUnknown = service::TextService::new().into();
        unsafe { ts.query(riid, ppv).ok() }
    }
    fn LockServer(&self, lock: BOOL) -> Result<()> {
        if lock.as_bool() {
            LOCKS.fetch_add(1, Ordering::Relaxed);
        } else {
            LOCKS.fetch_sub(1, Ordering::Relaxed);
        }
        Ok(())
    }
}

#[no_mangle]
extern "system" fn DllGetClassObject(rclsid: *const GUID, riid: *const GUID, ppv: *mut *mut c_void) -> HRESULT {
    unsafe {
        if *rclsid != CLSID_ARUARU_IME {
            return CLASS_E_CLASSNOTAVAILABLE;
        }
        let f: IClassFactory = Factory.into();
        f.query(riid, ppv)
    }
}

#[no_mangle]
extern "system" fn DllCanUnloadNow() -> HRESULT {
    // テキストサービスはホストプロセス内に常駐するため、アンロードは不可とする。
    S_FALSE
}

fn clsid_key() -> String {
    format!("SOFTWARE\\Classes\\CLSID\\{{{:?}}}", CLSID_ARUARU_IME).replace("{{", "{").replace("}}", "}")
}

fn set_reg(sub: &str, name: PCWSTR, value: &str) -> Result<()> {
    let sub_w: Vec<u16> = sub.encode_utf16().chain(Some(0)).collect();
    let val: Vec<u16> = value.encode_utf16().chain(Some(0)).collect();
    let bytes = unsafe { std::slice::from_raw_parts(val.as_ptr() as *const u8, val.len() * 2) };
    unsafe { RegSetKeyValueW(HKEY_LOCAL_MACHINE, PCWSTR(sub_w.as_ptr()), name, REG_SZ.0, Some(bytes.as_ptr() as *const c_void), bytes.len() as u32).ok() }
}

fn register() -> Result<()> {
    let key = clsid_key();
    set_reg(&key, PCWSTR::null(), DESCRIPTION)?;
    set_reg(&format!("{key}\\InprocServer32"), PCWSTR::null(), &dll_path().display().to_string())?;
    set_reg(&format!("{key}\\InprocServer32"), w!("ThreadingModel"), "Apartment")?;

    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let mgr: ITfInputProcessorProfileMgr = CoCreateInstance(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER)?;
        let desc: Vec<u16> = DESCRIPTION.encode_utf16().collect();
        let icon: Vec<u16> = dll_path().display().to_string().encode_utf16().collect();
        mgr.RegisterProfile(&CLSID_ARUARU_IME, LANGID_JA, &GUID_PROFILE, &desc, &icon, 0, HKL::default(), 0, true, 0)?;

        let cat: ITfCategoryMgr = CoCreateInstance(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER)?;
        for g in [
            GUID_TFCAT_TIP_KEYBOARD,
            GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT,
            GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT,
            GUID_TFCAT_TIPCAP_UIELEMENTENABLED,
            GUID_TFCAT_TIPCAP_COMLESS,
        ] {
            cat.RegisterCategory(&CLSID_ARUARU_IME, &g, &CLSID_ARUARU_IME)?;
        }
    }
    Ok(())
}

fn unregister() -> Result<()> {
    unsafe {
        let _ = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        if let Ok(mgr) = CoCreateInstance::<_, ITfInputProcessorProfileMgr>(&CLSID_TF_InputProcessorProfiles, None, CLSCTX_INPROC_SERVER) {
            let _ = mgr.UnregisterProfile(&CLSID_ARUARU_IME, LANGID_JA, &GUID_PROFILE, 0);
        }
        if let Ok(cat) = CoCreateInstance::<_, ITfCategoryMgr>(&CLSID_TF_CategoryMgr, None, CLSCTX_INPROC_SERVER) {
            for g in [GUID_TFCAT_TIP_KEYBOARD, GUID_TFCAT_TIPCAP_IMMERSIVESUPPORT, GUID_TFCAT_TIPCAP_SYSTRAYSUPPORT, GUID_TFCAT_TIPCAP_UIELEMENTENABLED, GUID_TFCAT_TIPCAP_COMLESS] {
                let _ = cat.UnregisterCategory(&CLSID_ARUARU_IME, &g, &CLSID_ARUARU_IME);
            }
        }
        let sub: Vec<u16> = clsid_key().encode_utf16().chain(Some(0)).collect();
        let _ = RegDeleteTreeW(HKEY_LOCAL_MACHINE, PCWSTR(sub.as_ptr()));
    }
    Ok(())
}

#[no_mangle]
extern "system" fn DllRegisterServer() -> HRESULT {
    match register() {
        Ok(()) => S_OK,
        Err(e) => e.code(),
    }
}

#[no_mangle]
extern "system" fn DllUnregisterServer() -> HRESULT {
    match unregister() {
        Ok(()) => S_OK,
        Err(e) => e.code(),
    }
}

/// 診断ログ(%TEMP%\aruaru-ime-tsf.log)。環境変数 ARUARU_IME_LOG=1 のときだけ書く。
pub fn log(msg: &str) {
    if std::env::var("ARUARU_IME_LOG").as_deref() != Ok("1") && !std::path::Path::new(&std::env::temp_dir().join("aruaru-ime-log.on")).exists() {
        return;
    }
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true).open(std::env::temp_dir().join("aruaru-ime-tsf.log")) {
        let _ = writeln!(f, "{:?} pid={} {}", std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis()).unwrap_or(0), std::process::id(), msg);
    }
}
