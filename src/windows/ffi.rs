use windows::{core::PCWSTR, Win32::Foundation::HMODULE};

#[link(name = "kernel32")]
extern "C" {
    pub fn LoadLibraryW(filename: PCWSTR) -> HMODULE;
    pub fn LoadLibraryExW(filename: PCWSTR, hfile: *mut std::ffi::c_void, flags: u32) -> HMODULE;
}