use crate::il2cpp::{
    symbols::{get_field_from_name, get_method_addr},
    types::*,
};

static mut GET_GATE_NO_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_GateNo, GET_GATE_NO_ADDR, i32, this: *mut Il2CppObject);

def_field_value_accessors!(get get__Popularity, POPULARITY_FIELD, i32);
def_field_object_accessors!(get get__responseHorseData, RESPONSE_HORSE_DATA_FIELD, Il2CppObject);
def_field_object_accessors!(get get__charaName, CHARA_NAME_FIELD, Il2CppString);
def_field_value_accessors!(get get__charaId, CHARA_ID_FIELD, i32);
def_field_value_accessors!(get get__Defeat, DEFEAT_FIELD, i32);

static mut GET_IS_USER_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_IsUser, GET_IS_USER_ADDR, bool, this: *mut Il2CppObject);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, HorseData);

    unsafe {
        GET_GATE_NO_ADDR = get_method_addr(HorseData, c"get_GateNo", 0);
        POPULARITY_FIELD = get_field_from_name(HorseData, c"<Popularity>k__BackingField");
        RESPONSE_HORSE_DATA_FIELD = get_field_from_name(HorseData, c"_responseHorseData");
        CHARA_NAME_FIELD = get_field_from_name(HorseData, c"<charaName>k__BackingField");
        CHARA_ID_FIELD = get_field_from_name(HorseData, c"charaId");
        DEFEAT_FIELD = get_field_from_name(HorseData, c"<Defeat>k__BackingField");
        GET_IS_USER_ADDR = get_method_addr(HorseData, c"get_IsUser", 0);
    }
}
