use std::{
    collections::HashMap,
    ffi::CStr,
    sync::{atomic::{AtomicU64, Ordering}, Mutex},
};

use jni::{
    objects::{GlobalRef, JClass, JMap, JObject, JValue},
    JNIEnv,
};
use once_cell::sync::Lazy;

use crate::android::main::java_vm;

fn log_exception(env: &mut JNIEnv, context: &str) {
    if env.exception_check().unwrap_or(false) {
        log::warn!("dex_bridge: JNI exception during {}", context);
        let _ = env.exception_describe();
        let _ = env.exception_clear();
    }
}

#[derive(Clone)]
struct DexEntry {
    #[allow(dead_code)]
    class_loader: GlobalRef,
    class_obj: GlobalRef,
}

static NEXT_HANDLE: AtomicU64 = AtomicU64::new(1);
static DEX_REGISTRY: Lazy<Mutex<HashMap<u64, DexEntry>>> = Lazy::new(|| Mutex::new(HashMap::new()));

fn get_activity(mut env: JNIEnv<'_>) -> Option<JObject<'_>> {
    // Try UnityPlayer.currentActivity first
    match env.find_class("com/unity3d/player/UnityPlayer") {
        Ok(unity_player_class) => {
            match env.get_static_field(unity_player_class, "currentActivity", "Landroid/app/Activity;") {
                Ok(val) => {
                    if let Ok(current_activity) = val.l() {
                        if !current_activity.is_null() {
                            log::debug!("dex_bridge: get_activity found via UnityPlayer.currentActivity");
                            return Some(current_activity);
                        }
                    }
                }
                Err(e) => {
                    log::debug!("dex_bridge: get_activity failed to get UnityPlayer.currentActivity: {:?}", e);
                }
            }
        }
        Err(e) => {
            log::debug!("dex_bridge: get_activity failed to find UnityPlayer class: {:?}", e);
        }
    }

    log::debug!("dex_bridge: get_activity trying ActivityThread fallback");
    
    // Try ActivityThread.currentActivityThread() fallback
    match env.find_class("android/app/ActivityThread") {
        Ok(activity_thread_class) => {
            match env.call_static_method(
                activity_thread_class,
                "currentActivityThread",
                "()Landroid/app/ActivityThread;",
                &[],
            ) {
                Ok(val) => {
                    if let Ok(activity_thread) = val.l() {
                        if !activity_thread.is_null() {
                            match env.get_field(activity_thread, "mActivities", "Landroid/util/ArrayMap;") {
                                Ok(activities_val) => {
                                    if let Ok(activities) = activities_val.l() {
                                        if !activities.is_null() {
                                            if let Ok(activities_map) = JMap::from_env(&mut env, &activities) {
                                                if let Ok(mut iter) = activities_map.iter(&mut env) {
                                                    while let Ok(Some((_, activity_record))) = iter.next(&mut env) {
                                                        if let Ok(activity_val) = env.get_field(activity_record, "activity", "Landroid/app/Activity;") {
                                                            if let Ok(activity) = activity_val.l() {
                                                                if !activity.is_null() {
                                                                    log::debug!("dex_bridge: get_activity found via mActivities");
                                                                    return Some(activity);
                                                                }
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                Err(e) => {
                                    log::debug!("dex_bridge: get_activity failed to get mActivities: {:?}", e);
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    log::debug!("dex_bridge: get_activity failed to call currentActivityThread: {:?}", e);
                }
            }
        }
        Err(e) => {
            log::debug!("dex_bridge: get_activity failed to find ActivityThread class: {:?}", e);
        }
    }

    log::warn!("dex_bridge: get_activity failed to retrieve Activity from any source");
    None
}

fn load_class_from_dex(env: &mut JNIEnv, dex_bytes: &[u8], class_name: &str) -> Option<(GlobalRef, GlobalRef)> {
    let activity = match get_activity(unsafe { env.unsafe_clone() }) {
        Some(activity) => activity,
        None => {
            log::error!("dex_bridge: No Activity found during cold initialization");
            return None;
        }
    };
    
    let class_loader = match env.call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[]) {
        Ok(val) => match val.l() {
            Ok(loader) => loader,
            Err(e) => {
                log::error!("dex_bridge: Failed to get ClassLoader object: {:?}", e);
                return None;
            }
        },
        Err(e) => {
            log::error!("dex_bridge: Failed to call getClassLoader: {:?}", e);
            return None;
        }
    };

    let byte_array = match env.byte_array_from_slice(dex_bytes) {
        Ok(arr) => arr,
        Err(e) => {
            log::error!("dex_bridge: Failed to create byte array: {:?}", e);
            return None;
        }
    };
    
    let byte_buffer = match env.call_static_method(
        "java/nio/ByteBuffer",
        "wrap",
        "([B)Ljava/nio/ByteBuffer;",
        &[JValue::Object(&JObject::from(byte_array))],
    ) {
        Ok(val) => match val.l() {
            Ok(buf) => buf,
            Err(e) => {
                log::error!("dex_bridge: Failed to get ByteBuffer object: {:?}", e);
                return None;
            }
        },
        Err(e) => {
            log::error!("dex_bridge: Failed to call ByteBuffer.wrap: {:?}", e);
            return None;
        }
    };

    let dex_loader = match env.new_object(
        "dalvik/system/InMemoryDexClassLoader",
        "(Ljava/nio/ByteBuffer;Ljava/lang/ClassLoader;)V",
        &[JValue::Object(&byte_buffer), JValue::Object(&class_loader)],
    ) {
        Ok(loader) => loader,
        Err(e) => {
            log::error!("dex_bridge: Failed to create InMemoryDexClassLoader: {:?}", e);
            return None;
        }
    };

    let class_name_str = match env.new_string(class_name) {
        Ok(s) => s,
        Err(e) => {
            log::error!("dex_bridge: Failed to create class name string: {:?}", e);
            return None;
        }
    };
    
    let class_obj = match env.call_method(
        &dex_loader,
        "loadClass",
        "(Ljava/lang/String;)Ljava/lang/Class;",
        &[JValue::Object(&class_name_str)],
    ) {
        Ok(val) => match val.l() {
            Ok(obj) => obj,
            Err(e) => {
                log::error!("dex_bridge: Failed to get loaded Class object: {:?}", e);
                return None;
            }
        },
        Err(e) => {
            log::error!("dex_bridge: Failed to call loadClass({}): {:?}", class_name, e);
            return None;
        }
    };

    let loader_ref = match env.new_global_ref(&dex_loader) {
        Ok(r) => r,
        Err(e) => {
            log::error!("dex_bridge: Failed to create global ref for loader: {:?}", e);
            return None;
        }
    };
    
    let class_ref = match env.new_global_ref(class_obj) {
        Ok(r) => r,
        Err(e) => {
            log::error!("dex_bridge: Failed to create global ref for class: {:?}", e);
            return None;
        }
    };
    
    log::info!("dex_bridge: Successfully loaded class {} from dex", class_name);
    Some((loader_ref, class_ref))
}

fn with_env<F: FnOnce(&mut JNIEnv) -> bool>(f: F) -> bool {
    let Some(vm) = java_vm() else { 
        log::error!("dex_bridge: JavaVM not available");
        return false; 
    };
    let Ok(mut env) = vm.attach_current_thread_as_daemon() else { 
        log::error!("dex_bridge: Failed to attach to JNI thread");
        return false; 
    };
    f(&mut env)
}

pub fn dex_load(dex_ptr: *const u8, dex_len: usize, class_name: *const std::os::raw::c_char) -> u64 {
    if dex_ptr.is_null() || dex_len == 0 {
        return 0;
    }
    if class_name.is_null() {
        return 0;
    }
    let Ok(class_name) = unsafe { CStr::from_ptr(class_name) }.to_str() else { return 0; };

    let dex_bytes = unsafe { std::slice::from_raw_parts(dex_ptr, dex_len) };

    let mut handle_out = 0;
    let ok = with_env(|env| {
        let Some((loader_ref, class_ref)) = load_class_from_dex(env, dex_bytes, class_name) else {
            log_exception(env, "dex_load");
            return false;
        };
        let handle = NEXT_HANDLE.fetch_add(1, Ordering::Relaxed);
        DEX_REGISTRY.lock().unwrap().insert(
            handle,
            DexEntry { class_loader: loader_ref, class_obj: class_ref },
        );
        handle_out = handle;
        true
    });

    if ok { handle_out } else { 0 }
}

pub fn dex_unload(handle: u64) -> bool {
    DEX_REGISTRY.lock().unwrap().remove(&handle).is_some()
}

pub fn call_static_noargs(handle: u64, method: &CStr, sig: &CStr) -> bool {
    let Ok(method) = method.to_str() else { return false; };
    let Ok(sig) = sig.to_str() else { return false; };
    let entry = DEX_REGISTRY.lock().unwrap().get(&handle).cloned();
    let Some(entry) = entry else { return false; };

    with_env(|env| {
        let Ok(class_obj) = env.new_local_ref(entry.class_obj.as_obj()) else { return false; };
        let class = JClass::from(class_obj);
        match env.call_static_method(class, method, sig, &[]) {
            Ok(_) => true,
            Err(_) => {
                log::warn!("dex_bridge: call_static_noargs failed ({} {})", method, sig);
                log_exception(env, "call_static_noargs");
                false
            }
        }
    })
}

pub fn call_static_string(handle: u64, method: &CStr, sig: &CStr, arg: &CStr) -> bool {
    let Ok(method) = method.to_str() else { return false; };
    let Ok(sig) = sig.to_str() else { return false; };
    let Ok(arg_str) = arg.to_str() else { return false; };
    let entry = DEX_REGISTRY.lock().unwrap().get(&handle).cloned();
    let Some(entry) = entry else { return false; };

    with_env(|env| {
        let Ok(class_obj) = env.new_local_ref(entry.class_obj.as_obj()) else { return false; };
        let class = JClass::from(class_obj);
        let Ok(jarg) = env.new_string(arg_str) else { return false; };
        match env.call_static_method(class, method, sig, &[JValue::Object(&jarg)]) {
            Ok(_) => true,
            Err(_) => {
                log::warn!("dex_bridge: call_static_string failed ({} {})", method, sig);
                log_exception(env, "call_static_string");
                false
            }
        }
    })
}
