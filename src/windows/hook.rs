#![allow(non_snake_case)]

use std::ffi::c_void;
use std::path::Path;

use windows::{core::{w, PCWSTR}, Win32::{Foundation::HMODULE, System::LibraryLoader::GetModuleHandleW}};

use crate::{core::{Error, Hachimi}, windows::{steamworks, utils}};

use super::{hachimi_impl, proxy, ffi};

fn handle_loaded_library(filename_str: &str, handle: HMODULE) {
    let hachimi = Hachimi::instance();

    if hachimi_impl::is_criware_lib(filename_str) {
        // Manually trigger a GameAssembly.dll load anyways since hachimi might have been loaded later
        if let Ok(assembly_handle) = unsafe { GetModuleHandleW(w!("GameAssembly.dll")) } {
            let assembly_module = assembly_handle.0 as usize;
            if assembly_module != 0 {
                hachimi.on_dlopen("GameAssembly.dll", assembly_module);
            }
        }
    }

    let needs_init_steamworks = steamworks::needs_init(&hachimi);
    if hachimi.on_dlopen(filename_str, handle.0 as usize) {
        if !needs_init_steamworks {
            unhook_library_loaders();
        }
    }
    else if needs_init_steamworks &&
        Path::new(filename_str).file_name().is_some_and(|name| name == "steam_api64.dll")
    {
        steamworks::init(handle);
        unhook_library_loaders();
    }
}

fn unhook_library_loaders() {
    let hachimi = Hachimi::instance();
    hachimi.interceptor.unhook(LoadLibraryW as *const () as usize);
    hachimi.interceptor.unhook(LoadLibraryExW as *const () as usize);
}

type LoadLibraryWFn = extern "C" fn(filename: PCWSTR) -> HMODULE;
extern "C" fn LoadLibraryW(filename: PCWSTR) -> HMODULE {
    let hachimi = Hachimi::instance();
    let trampoline = hachimi.interceptor.get_trampoline_addr(LoadLibraryW as *const () as usize);
    if trampoline == 0 {
        return unsafe { crate::windows::ffi::LoadLibraryW(filename) };
    }
    let orig_fn: LoadLibraryWFn = unsafe { std::mem::transmute(trampoline) };

    let handle = orig_fn(filename);
    let filename_str = match unsafe { filename.to_string() } {
        Ok(s) => s,
        Err(_) => return handle,
    };

    handle_loaded_library(&filename_str, handle);
    handle
}

type LoadLibraryExWFn = extern "C" fn(filename: PCWSTR, file: *mut c_void, flags: u32) -> HMODULE;
extern "C" fn LoadLibraryExW(filename: PCWSTR, file: *mut c_void, flags: u32) -> HMODULE {
    let hachimi = Hachimi::instance();
    let trampoline = hachimi.interceptor.get_trampoline_addr(LoadLibraryExW as *const () as usize);
    if trampoline == 0 {
        return unsafe { crate::windows::ffi::LoadLibraryExW(filename, file, flags) };
    }
    let orig_fn: LoadLibraryExWFn = unsafe { std::mem::transmute(trampoline) };

    let handle = orig_fn(filename, file, flags);
    let filename_str = match unsafe { filename.to_string() } {
        Ok(s) => s,
        Err(_) => return handle,
    };

    handle_loaded_library(&filename_str, handle);
    handle
}

fn init_internal() -> Result<(), Error> {
    let hachimi = Hachimi::instance();

    // UnityPlayer proxy exports UnityMain, so it must always be initialized
    // to provide valid forwarding targets regardless of loader method.
    info!("Init UnityPlayer.dll proxy exports");
    proxy::unityplayer::init();

    let dll_module = unsafe { super::main::DLL_HMODULE };
    let module_name = utils::get_module_path(dll_module)
        .file_name().map(|s| s.to_string_lossy().to_ascii_lowercase());

    match module_name.as_deref() {
        Some("winhttp.dll") => {
            info!("Init winhttp.dll proxy");
            proxy::winhttp::init(&utils::_get_system_directory());
        }
        Some("cri_mana_vpx.dll") => {
            info!("Init cri_mana_vpx.dll proxy");
            proxy::cri_mana_vpx::init();
        }
        _ => {}
    }

    info!("Hooking LoadLibraryW and LoadLibraryExW");
    hachimi.interceptor.hook(ffi::LoadLibraryW as *const () as usize, LoadLibraryW as *const () as usize)?;
    hachimi.interceptor.hook(ffi::LoadLibraryExW as *const () as usize, LoadLibraryExW as *const () as usize)?;

    if let Ok(handle) = unsafe { GetModuleHandleW(w!("GameAssembly.dll")) } {
        if !handle.is_invalid() && handle.0 as usize != 0 {
            info!("Late loading detected");
            hachimi.on_dlopen("GameAssembly.dll", handle.0 as _);
            hachimi.on_hooking_finished();
        }
    }

    Ok(())
}

pub fn init() {
    init_internal().unwrap_or_else(|e| {
        error!("Init failed: {}", e);
    });
}