macro_rules! proxy_proc {
    ($name:ident, $orig_var_name:ident) => {
        static mut $orig_var_name: usize = 0;
        std::arch::global_asm!(
            concat!(".globl ", stringify!($name)),
            concat!(stringify!($name), ":"),
            // Forward to the real DLL when it was loaded. If it wasn't (e.g. the
            // proxy init skipped because the real DLL is missing), return NULL
            // instead of jumping through a zeroed pointer (which crashes the
            // caller with an execute-access fault at address 0).
            "    mov rax, qword ptr [rip + {}]",
            "    test rax, rax",
            "    jnz 1f",
            "    xor eax, eax",
            "    ret",
            "1:",
            "    jmp rax",
            sym $orig_var_name
        );
    }
}

pub mod cri_mana_vpx;
pub mod unityplayer;
pub mod winhttp;