use std::{ffi::*, iter, mem, ptr::*};

use maya_sys::root::Autodesk::Maya::OpenMaya20180000 as OpenMaya;
use retour::static_detour;
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::LibraryLoader::{GetModuleHandleW, GetProcAddress};
use windows::Win32::System::SystemServices::{
    DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH, DLL_THREAD_ATTACH, DLL_THREAD_DETACH,
};
use windows::Win32::UI::WindowsAndMessaging::*;
use windows::core::{BOOL, PCSTR, PCWSTR, s};

const SHARED_LIBRARY_NAME: &str = "Shared.dll";
const TDN_EXECUTE_BEFORE_SCRIPT_SYMBOL: &str = "?executeBeforeScript@TdnScriptNode@@QEAA_NXZ";
const TDN_EXECUTE_AFTER_SCRIPT_SYMBOL: &str = "?executeAfterScript@TdnScriptNode@@QEAA_NXZ";

type FnAPIVersion = unsafe extern "C" fn(Option<NonNull<c_void>>) -> c_int;
type FnTdnExecuteScriptNode = unsafe extern "C" fn(c_void) -> c_int;

static_detour!(
    static APIVersionHook: unsafe extern "C" fn(Option<NonNull<c_void>>) -> c_int;
    static TdnScriptBeforeExecuteScriptNodeHook: unsafe extern "C" fn(c_void) -> c_int;
    static TdnScriptAfterExecuteScriptNodeHook: unsafe extern "C" fn(c_void) -> c_int;
);

static mut TDN_EXECUTE_BEFORE_SCRIPT_FUNC_ADDRESS: usize = 0;
static mut TDN_EXECUTE_AFTER_SCRIPT_FUNC_ADDRESS: usize = 0;
static mut API_VERSION_FUNC_ADDRESS: usize = 0;

fn initialize_address() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        // use maya-sys get address
        API_VERSION_FUNC_ADDRESS = get_real_address(OpenMaya::MGlobal_apiVersion as usize);

        // dynamic get address
        TDN_EXECUTE_BEFORE_SCRIPT_FUNC_ADDRESS =
            get_module_symbol_address(SHARED_LIBRARY_NAME, TDN_EXECUTE_BEFORE_SCRIPT_SYMBOL)
                .ok_or("Failed to get TDN_EXECUTE_BEFORE_SCRIPT_FUNC_ADDRESS")?;
        TDN_EXECUTE_AFTER_SCRIPT_FUNC_ADDRESS =
            get_module_symbol_address(SHARED_LIBRARY_NAME, TDN_EXECUTE_AFTER_SCRIPT_SYMBOL)
                .ok_or("Failed to get TDN_EXECUTE_AFTER_SCRIPT_FUNC_ADDRESS")?;
    }

    Ok(())
}

fn api_version_hook(_: Option<NonNull<c_void>>) -> c_int {
    // you can add some code here or change the return value
    // unsafe { APIVersionHook.call(option) }

    // fake api version
    c_int::from(20990000)
}

fn tdn_script_before_execute_script_node_hook(_script_node: c_void) -> c_int {
    // ignore all script
    // unsafe { TdnScriptBeforeExecuteScriptNodeHook.call(script_node) }

    display_message("[MayaX] Success ignored beforeScript execution\0");

    // fake return
    c_int::from(true)
}

fn tdn_script_after_execute_script_node_hook(_script_node: c_void) -> c_int {
    // ignore all script
    // unsafe { TdnScriptAfterExecuteScriptNodeHook.call(script_node) }

    display_message("[MayaX] Success ignored afterScript execution\0");

    // fake return
    c_int::from(true)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    display_message("[MayaX] Initializing Runtime...\0");

    initialize_address()?;
    display_message("[MayaX] Initialized Hook Function Address\0");

    // initial hooks
    unsafe {
        let func: FnAPIVersion = mem::transmute(API_VERSION_FUNC_ADDRESS);
        APIVersionHook
            .initialize(func, api_version_hook)?
            .enable()?;

        let func: FnTdnExecuteScriptNode = mem::transmute(TDN_EXECUTE_BEFORE_SCRIPT_FUNC_ADDRESS);
        TdnScriptBeforeExecuteScriptNodeHook
            .initialize(func, tdn_script_before_execute_script_node_hook)?
            .enable()?;

        let func: FnTdnExecuteScriptNode = mem::transmute(TDN_EXECUTE_AFTER_SCRIPT_FUNC_ADDRESS);
        TdnScriptAfterExecuteScriptNodeHook
            .initialize(func, tdn_script_after_execute_script_node_hook)?
            .enable()?;
    }
    display_message("[MayaX] Initialized Runtime\0");

    Ok(())
}

fn display_message(msg: &str) {
    unsafe {
        let msg = OpenMaya::MString::new1(msg.as_ptr() as _);
        OpenMaya::MGlobal::displayInfo(&msg);
    }
}

fn get_real_address(base_address: usize) -> usize {
    unsafe {
        let offset_address = (base_address as *const u8).add(2) as *const u32;
        let code = *offset_address;
        let address = ((base_address as usize) + 0x06 + code as usize) as *const usize;
        let address = *address;
        address
    }
}

fn get_module_symbol_address(module: &str, symbol: &str) -> Option<usize> {
    let module = module
        .encode_utf16()
        .chain(iter::once(0))
        .collect::<Vec<u16>>();
    let symbol = CString::new(symbol).unwrap();
    unsafe {
        let handle = GetModuleHandleW(PCWSTR(module.as_ptr() as _)).unwrap();
        match GetProcAddress(handle, PCSTR(symbol.as_ptr() as _)) {
            Some(func) => Some(func as usize),
            None => None,
        }
    }
}

#[unsafe(no_mangle)]
extern "system" fn DllMain(_dll_module: HANDLE, call_reason: u32, _reserved: *mut c_void) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => match main() {
            Err(e) => unsafe {
                MessageBoxA(
                    None,
                    PCSTR(e.to_string().as_ptr() as _),
                    s!("MayaX Runtime Error"),
                    MB_OK,
                );
            },
            Ok(_) => unsafe {
                MessageBoxA(None, s!("MayaX Runtime Injected"), s!("MayaX"), MB_OK);
            },
        },
        DLL_PROCESS_DETACH => {}
        DLL_THREAD_ATTACH => {}
        DLL_THREAD_DETACH => {}
        _ => {}
    }

    BOOL::from(true)
}
