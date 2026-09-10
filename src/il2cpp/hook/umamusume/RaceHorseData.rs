// Gallop.RaceHorseData lives in umamusume.Http.dll, not umamusume.dll like most Gallop.*
// classes - confirmed against a JP il2cpp dump by the race-director-plugin this module
// was ported from.
use crate::il2cpp::{
    symbols::get_field_from_name,
    types::*,
};

def_field_value_accessors!(get get__runningStyle, RUNNING_STYLE_FIELD, i32);
def_field_object_accessors!(get get__trainerName, TRAINER_NAME_FIELD, Il2CppString);
def_field_value_accessors!(get get__viewerId, VIEWER_ID_FIELD, i64);

pub fn init(_umamusume: *const Il2CppImage) {
    get_assembly_image_or_return!(http_image, "umamusume.Http.dll");
    get_class_or_return!(http_image, Gallop, RaceHorseData);

    unsafe {
        RUNNING_STYLE_FIELD = get_field_from_name(RaceHorseData, c"running_style");
        TRAINER_NAME_FIELD = get_field_from_name(RaceHorseData, c"trainer_name");
        VIEWER_ID_FIELD = get_field_from_name(RaceHorseData, c"viewer_id");
    }
}
