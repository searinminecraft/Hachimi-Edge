use std::sync::atomic::Ordering;

use crate::{core::{utils, Hachimi}, il2cpp::{api::{il2cpp_class_get_type, il2cpp_type_get_object}, ext::{Il2CppObjectExt, Il2CppStringExt, LocalizedDataExt}, hook::{UnityEngine_TextRenderingModule::TextGenerator::mark_as_system_text_component, UnityEngine_UI::Text}, sql::IS_SYSTEM_TEXT_QUERY, symbols::get_method_addr, types::*}};

static mut TYPE_OBJECT: *mut Il2CppObject = 0 as _;
pub fn type_object() -> *mut Il2CppObject {
    unsafe { TYPE_OBJECT }
}

static mut GET_IS_ACTIVE_IN_HIERARCHY_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_IsActiveInHierarchy, GET_IS_ACTIVE_IN_HIERARCHY_ADDR, bool, this: *mut Il2CppObject);

type AwakeFn = extern "C" fn(this: *mut Il2CppObject);
extern "C" fn Awake(this: *mut Il2CppObject) {
    get_orig_fn!(Awake, AwakeFn)(this);

    let localized_data = Hachimi::instance().localized_data.load();
    let config = Hachimi::instance().config.load();

    // Replace to built-in font if configured (Edge addition)
    if config.replace_to_builtin_font {
        unsafe {
            let assign_default_font = crate::il2cpp::symbols::get_method_addr_cached((*this).klass(), c"AssignDefaultFont", 0);
            if assign_default_font != 0 {
                let func: extern "C" fn(*mut Il2CppObject) = std::mem::transmute(assign_default_font);
                func(this);
            }
        }
    }

    let font = localized_data.load_replacement_font();
    if !font.is_null() {
        Text::set_font(this, font);
    }

    // kairusds granularity (config-gated, RekoChimi backend): enable best-fit on
    // components the game didn't already set, using per-instance size bounds.
    if localized_data.config.text_common_best_fit {
        // Do not touch game-set instances as they likely use special values.
        if !Text::get_resizeTextForBestFit(this) {
            let cur_size = Text::get_fontSize(this);
            Text::set_resizeTextMinSize(this, cur_size.min(10));
            Text::set_resizeTextMaxSize(this, cur_size);
            Text::set_resizeTextForBestFit(this, true);
        }
    }

    if localized_data.config.text_common_allow_overflow {
        Text::set_horizontalOverflow(this, 1);
        Text::set_verticalOverflow(this, 1);
    }
}

// We make the assumption the basic process of these functions is to call
// GallopUtil::LineHeadWrapForSystemText and set_text() the return value.
// The presumed reason those are not called directly is special handling and TextCommon
// object adjustments, which is exactly what we'll do here and take over wrapping.

/// Sets IS_SYSTEM_TEXT_QUERY for the duration of the callback so GallopUtil
/// knows to skip its own wrapping on text that we're already handling.
fn with_system_text_query(callback: impl FnOnce()) {
    IS_SYSTEM_TEXT_QUERY.store(true, Ordering::Relaxed);
    callback();
    IS_SYSTEM_TEXT_QUERY.store(false, Ordering::Relaxed);
}

// force best fit and wrap for system text (RekoChimi base behavior)
fn force_best_fit(this: *mut Il2CppObject) {
    Text::set_horizontalOverflow(this, 0);
    Text::set_resizeTextForBestFit(this, true);
    Text::set_resizeTextMinSize(this, 14);
    Text::set_resizeTextMaxSize(this, 30);
}

type SetSystemTextWithLineHeadWrapFn = extern "C" fn(this: *mut Il2CppObject, system_text: *mut CharacterSystemText, maxCharacter: i32);
extern "C" fn SetSystemTextWithLineHeadWrap(this: *mut Il2CppObject, system_text: *mut CharacterSystemText, max_character: i32) {
    unsafe { mark_as_system_text_component(this); }

    // kairusds granularity (ported onto RekoChimi subsystem): for characters
    // with localized system text, wrap by cue-sheet line limit and let the
    // wrapper dictate display instead of the game's layout pass.
    let localized_data = Hachimi::instance().localized_data.load();
    let systext = unsafe { &*system_text };
    let is_localized = localized_data.character_system_text_dict
        .get(&systext.characterId)
        .and_then(|c| c.get(&systext.voiceId))
        .is_some();
    if is_localized {
        let cue_sheet = unsafe { (*systext.cueSheet).as_utf16str().to_string() };
        let cue_type = cue_sheet.split('_').nth(2).unwrap_or_default();
        let font_size = Text::get_fontSize(this);
        let max_lines = *localized_data.config.systext_cue_lines.get(cue_type)
            .unwrap_or_else(|| localized_data.config.systext_cue_lines.get("default").unwrap_or(&4));

        if let Some(wrapped_text) = unsafe { utils::wrap_fit_text_il2cpp(systext.text, max_character, max_lines, font_size) } {
            // Allow wrapper to dictate display.
            Text::set_horizontalOverflow(this, 1);
            Text::set_verticalOverflow(this, 1);
            return Text::set_text(this, wrapped_text);
        }
    }

    // force best fit and wrap for system text
    force_best_fit(this);

    with_system_text_query(|| {
        get_orig_fn!(SetSystemTextWithLineHeadWrap, SetSystemTextWithLineHeadWrapFn)(this, system_text, max_character);
    });
}

type SetTextWithLineHeadWrapFn = extern "C" fn(this: *mut Il2CppObject, str: *mut Il2CppString, maxCharacter: i32);
extern "C" fn SetTextWithLineHeadWrap(this: *mut Il2CppObject, str: *mut Il2CppString, max_character: i32) {
    unsafe { mark_as_system_text_component(this); }

    // force best fit and wrap for system text
    force_best_fit(this);

    with_system_text_query(|| {
        get_orig_fn!(SetTextWithLineHeadWrap, SetTextWithLineHeadWrapFn)(this, str, max_character);
    });
}

type SetTextWithLineHeadWrapWithColorTagFn = extern "C" fn(this: *mut Il2CppObject, str: *mut Il2CppString, maxCharacter: i32);
extern "C" fn SetTextWithLineHeadWrapWithColorTag(this: *mut Il2CppObject, str: *mut Il2CppString, max_character: i32) {
    unsafe { mark_as_system_text_component(this); }

    // force best fit and wrap for system text
    force_best_fit(this);

    with_system_text_query(|| {
        get_orig_fn!(SetTextWithLineHeadWrapWithColorTag, SetTextWithLineHeadWrapWithColorTagFn)(this, str, max_character);
    });
}

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, TextCommon);

    let Awake_addr = get_method_addr(TextCommon, c"Awake", 0);
    new_hook!(Awake_addr, Awake);

    let SetSystemTextWithLineHeadWrap_addr = get_method_addr(TextCommon, c"SetSystemTextWithLineHeadWrap", 2);
    new_hook!(SetSystemTextWithLineHeadWrap_addr, SetSystemTextWithLineHeadWrap);

    let SetTextWithLineHeadWrap_addr = get_method_addr(TextCommon, c"SetTextWithLineHeadWrap", 2);
    new_hook!(SetTextWithLineHeadWrap_addr, SetTextWithLineHeadWrap);

    let SetTextWithLineHeadWrapWithColorTag_addr = get_method_addr(TextCommon, c"SetTextWithLineHeadWrapWithColorTag", 2);
    new_hook!(SetTextWithLineHeadWrapWithColorTag_addr, SetTextWithLineHeadWrapWithColorTag);

    unsafe {
        TYPE_OBJECT = il2cpp_type_get_object(il2cpp_class_get_type(TextCommon));
        GET_IS_ACTIVE_IN_HIERARCHY_ADDR = get_method_addr(TextCommon, c"get_IsActiveInHierarchy", 0);
    }
}
