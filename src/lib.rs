use memory_utils::MemoryUtils;
use std::{
    ffi::{c_void, CStr},
    os::raw::c_char,
};
use torchlight_utils::TorchlightUtils;
mod memory_utils;
mod torchlight_utils;

//定义全局变量
static mut TORCHLIGHT_UTILS: Option<&mut TorchlightUtils> = None;

#[no_mangle]
pub extern "C" fn init_utils(
    window_class: *const c_char,
    window_name: *const c_char,
) -> *mut c_void {
    let class_slice = unsafe { CStr::from_ptr(window_name) };
    //println!("window_class string buffer size without nul terminator: {}", class_slice.to_bytes().len());
    let class = class_slice.to_str().unwrap_or_else(|err| "");
    println!("window_class:{}", class);

    let name_slice = unsafe { CStr::from_ptr(window_class) };
    //println!("window_name string buffer size without nul terminator: {}", name_slice.to_bytes().len());
    let name = name_slice.to_str().unwrap_or_else(|err| "");
    println!("window_name:{}", name);

    // let class = unsafe { CStr::from_ptr(window_class).to_string_lossy().into_owned() };
    // println!("window_class:{}",class);

    // let name = unsafe { CStr::from_ptr(window_name).to_string_lossy().into_owned() };
    // println!("window_name:{}",name);

    if name == "" || class == "" {
        println!("window_class or window_name can not be null.");
        //return false;
    }

    let memory_utils = MemoryUtils::new(class, name);
    let  torchlight_utils = TorchlightUtils::new(memory_utils);
    println!("torchlight_utils:{:#?}", torchlight_utils);
    let torchlight_utils_box = Box::new(torchlight_utils);
    unsafe {
        TORCHLIGHT_UTILS = Some(Box::leak(torchlight_utils_box));
        //传递的是option的指针
        &mut TORCHLIGHT_UTILS as *mut Option<&'static mut TorchlightUtils<'static>> as *mut c_void
    }
    //&mut torchlight_utils as *mut TorchlightUtils as *mut c_void
    //Option<&'static mut TorchlightUtils<'static>>
}

#[no_mangle]
pub extern "C" fn hp_infinite(torchlight_utils: *mut c_void, new_hp: f32) -> bool {
    let utils = unsafe { &mut *(torchlight_utils as *mut Option<&'static mut TorchlightUtils<'static>>) };
    //println!("torchlight_utils:{:#?}",utils);
    //需要对option解引用
    utils.as_ref().unwrap().hp_infinite(new_hp)
}

#[no_mangle]
pub extern "C" fn mp_infinite(torchlight_utils: *mut c_void, new_mp: f32) -> bool {
    let utils = unsafe { &mut *(torchlight_utils as *mut Option<&'static mut TorchlightUtils<'static>>) };
    //println!("torchlight_utils:{:#?}",utils);
    //需要对option解引用
    utils.as_ref().unwrap().mp_infinite(new_mp)
}

#[no_mangle]
pub extern "C" fn gold_infinite(torchlight_utils: *mut c_void, new_gold: i32) -> bool {
    let utils = unsafe { &mut *(torchlight_utils as *mut Option<&'static mut TorchlightUtils<'static>>) };
    //println!("torchlight_utils:{:#?}",utils);
    //需要对option解引用
    utils.as_ref().unwrap().gold_infinite(new_gold)
}

#[no_mangle]
pub extern "C" fn add(a: i32, b: i32) -> i32 {
    a + b
}
