use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GET_IS_TEMPTATION_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_IsTemptation, GET_IS_TEMPTATION_ADDR, bool, this: *mut Il2CppObject);

static mut GET_TEMPTATION_MODE_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_TemptationMode, GET_TEMPTATION_MODE_ADDR, i32, this: *mut Il2CppObject);

static mut GET_POSITION_KEEP_MODE_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_PositionKeepMode, GET_POSITION_KEEP_MODE_ADDR, i32, this: *mut Il2CppObject);

static mut GET_IS_DOWN_SLOPE_ACCEL_MODE_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_IsDownSlopeAccelMode, GET_IS_DOWN_SLOPE_ACCEL_MODE_ADDR, bool, this: *mut Il2CppObject);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, HorseRaceAIBase);

    unsafe {
        GET_IS_TEMPTATION_ADDR = get_method_addr(HorseRaceAIBase, c"get_IsTemptation", 0);
        GET_TEMPTATION_MODE_ADDR = get_method_addr(HorseRaceAIBase, c"get_TemptationMode", 0);
        GET_POSITION_KEEP_MODE_ADDR = get_method_addr(HorseRaceAIBase, c"get_PositionKeepMode", 0);
        GET_IS_DOWN_SLOPE_ACCEL_MODE_ADDR = get_method_addr(HorseRaceAIBase, c"get_IsDownSlopeAccelMode", 0);
    }
}
