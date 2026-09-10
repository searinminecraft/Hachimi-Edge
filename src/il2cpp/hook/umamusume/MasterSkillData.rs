use crate::il2cpp::{symbols::get_field_from_name, symbols::get_method_addr, types::*};

// Gallop.MasterSkillData.Get(int) - an INSTANCE method (confirmed against the
// race-director-plugin's own live JP testing: it resolved the instance via
// MasterDataManager's masterSkillData field, not a static call), returning one
// Gallop.MasterSkillData.SkillData row. `this` = MasterDataManager::get__masterSkillData().
static mut GET_ADDR: usize = 0;
impl_addr_wrapper_fn!(Get, GET_ADDR, *mut Il2CppObject, this: *mut Il2CppObject, id: i32);

// Gallop.MasterSkillData.SkillData (the master DB row, NOT Gallop.SkillData - the
// wire/msgpack DTO with just skill_id/level). A nested type: reached via
// find_nested_class_or_return! rather than a direct namespace+name lookup.
def_field_value_accessors!(get get__abilityType11, ABILITY_TYPE11_FIELD, i32);
def_field_value_accessors!(get get__floatAbilityValue11, FLOAT_ABILITY_VALUE11_FIELD, i32);
def_field_value_accessors!(get get__floatAbilityTime1, FLOAT_ABILITY_TIME1_FIELD, i32);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, MasterSkillData);

    unsafe {
        GET_ADDR = get_method_addr(MasterSkillData, c"Get", 1);
    }

    find_nested_class_or_return!(MasterSkillData, SkillData);

    unsafe {
        ABILITY_TYPE11_FIELD = get_field_from_name(SkillData, c"AbilityType11");
        FLOAT_ABILITY_VALUE11_FIELD = get_field_from_name(SkillData, c"FloatAbilityValue11");
        FLOAT_ABILITY_TIME1_FIELD = get_field_from_name(SkillData, c"FloatAbilityTime1");
    }
}
