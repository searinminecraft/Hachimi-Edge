use crate::il2cpp::{symbols::get_method_addr, types::*};

static mut GET_LAST_SPURT_CALC_RESULT_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_LastSpurtCalcResult, GET_LAST_SPURT_CALC_RESULT_ADDR, i32, this: *mut Il2CppObject);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, HorseRaceAIReplay);

    unsafe {
        GET_LAST_SPURT_CALC_RESULT_ADDR = get_method_addr(HorseRaceAIReplay, c"get_LastSpurtCalcResult", 0);
    }
}
