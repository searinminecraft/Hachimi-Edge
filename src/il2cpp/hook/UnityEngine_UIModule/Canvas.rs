use crate::il2cpp::{api::{il2cpp_class_get_type, il2cpp_type_get_object}, symbols::get_method_addr, types::*};
use crate::il2cpp::hook::UnityEngine_TextRenderingModule::TextGenerator;
use log::error;

static mut TYPE_OBJECT: *mut Il2CppObject = 0 as _;
pub fn type_object() -> *mut Il2CppObject {
    unsafe { TYPE_OBJECT }
}

type SendWillRenderCanvasesFn = extern "C" fn();
extern "C" fn SendWillRenderCanvases() {
    // Call original hook
    if let Err(e) = std::panic::catch_unwind(|| {
        get_orig_fn!(SendWillRenderCanvases, SendWillRenderCanvasesFn)();
    }) {
        error!("SendWillRenderCanvases original hook panicked: {:?}", e);
    }

    // Apply any queued position offsets after layout pass
    if let Err(e) = std::panic::catch_unwind(|| {
        TextGenerator::drain_pending_offsets();
    }) {
        error!("TextGenerator::drain_pending_offsets panicked: {:?}", e);
    }
}

pub fn init(UnityEngine_UIModule: *const Il2CppImage) {
    get_class_or_return!(UnityEngine_UIModule, UnityEngine, Canvas);

    unsafe {
        TYPE_OBJECT = il2cpp_type_get_object(il2cpp_class_get_type(Canvas));
    }

    let SendWillRenderCanvases_addr = get_method_addr(Canvas, c"SendWillRenderCanvases", 0);
    new_hook!(SendWillRenderCanvases_addr, SendWillRenderCanvases);
}
