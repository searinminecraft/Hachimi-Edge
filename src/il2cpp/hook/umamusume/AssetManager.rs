use crate::il2cpp::{
    api::{il2cpp_class_get_method_from_name, il2cpp_runtime_invoke},
    ext::Il2CppObjectExt,
    symbols::{get_method_addr, get_assembly_image, get_class},
    types::*
};

static mut CLASS: *mut Il2CppClass = 0 as _;
pub fn class() -> *mut Il2CppClass {
    unsafe { CLASS }
}

static mut GET_LOADER_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_Loader, GET_LOADER_ADDR, *mut Il2CppObject,);

pub fn LoadAssetHandle(this: *mut Il2CppObject, path: *mut Il2CppString, flag: bool) -> *mut Il2CppObject {
    if this.is_null() { return std::ptr::null_mut(); }
    let class = unsafe { (*this).klass() };
    let method = il2cpp_class_get_method_from_name(class, c"LoadAssetHandle".as_ptr(), 2);
    if method.is_null() { return std::ptr::null_mut(); }
    let mut params: [*mut std::ffi::c_void; 2] = [path as *mut _, &flag as *const _ as *mut _];
    let mut exception = std::ptr::null_mut();
    let result = il2cpp_runtime_invoke(method, this as *mut _, params.as_mut_ptr(), &mut exception);
    if exception.is_null() { result } else { std::ptr::null_mut() }
}

static mut GET_ASSETBUNDLE_ADDR: usize = 0;
impl_addr_wrapper_fn!(get_assetBundle, GET_ASSETBUNDLE_ADDR, *mut Il2CppObject, this: *mut Il2CppObject);

pub fn init(umamusume: *const Il2CppImage) {
    get_class_or_return!(umamusume, Gallop, AssetManager);

    unsafe {
        CLASS = AssetManager;
        GET_LOADER_ADDR = get_method_addr(AssetManager, c"get_Loader", 0);
    }

    if let Ok(cyan_image) = get_assembly_image(c"_Cyan.dll") {
        if let Ok(asset_handle_class) = get_class(cyan_image, c"Cyan.Loader", c"AssetHandle") {
            unsafe {
                GET_ASSETBUNDLE_ADDR = get_method_addr(asset_handle_class, c"get_assetBundle", 0);
            }
        } else {
            error!("Failed to find AssetHandle class in _Cyan.dll");
        }
    } else {
        error!("Failed to load _Cyan.dll assembly");
    }
}
