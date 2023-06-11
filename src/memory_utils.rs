use std::{
    ffi::{c_void, OsStr},
    io,
    iter::once,
    mem::size_of,
    os::{raw::c_int, windows::prelude::OsStrExt},
    ptr,
};

use winapi::{
    ctypes::c_float,
    shared::{minwindef::FALSE, windef::HWND},
    um::{
        memoryapi::{ReadProcessMemory, WriteProcessMemory},
        processthreadsapi::OpenProcess,
        winnt::{HANDLE, PROCESS_ALL_ACCESS},
        winuser::{FindWindowA, FindWindowW, GetWindowThreadProcessId},
    },
};

#[derive(Debug)]
pub struct MemoryUtils<'a> {
    pub window_class: &'a str,
    pub window_name: &'a str,
    pub hwnd: HWND,             //窗口句柄
    pub pid: u32,               //进程id
    pub process_handle: HANDLE, //进程句柄
}

impl<'a> MemoryUtils<'a> {
    pub fn new(window_class: &'a str, window_name: &'a str) -> Self {
        let hwnd = MemoryUtils::get_window_handle(window_class, window_name);
        let pid = MemoryUtils::get_window_pid(hwnd);
        let process_handle = MemoryUtils::get_process_handle(pid);

        MemoryUtils {
            window_class,
            window_name,
            hwnd,
            pid,
            process_handle,
        }
    }

    //获取窗口句柄
    pub fn get_window_handle(window_class: &'a str, window_name: &'a str) -> HWND {
        let hwnd = unsafe {
            let window_class: Vec<u16> = OsStr::new(window_class)
                .encode_wide()
                .chain(once(0))
                .collect();
            let window_name: Vec<u16> = OsStr::new(window_name)
                .encode_wide()
                .chain(once(0))
                .collect();
            FindWindowW(window_class.as_ptr(), window_name.as_ptr())
        };
        println!("窗口句柄为：{:?}", hwnd);
        hwnd
    }

    //获取窗口进程id，也就是pid
    pub fn get_window_pid(hwnd: HWND) -> u32 {
        let mut pid = 0u32;
        unsafe {
            GetWindowThreadProcessId(hwnd, &mut pid);
        }
        println!("窗口进程id为：{:?}", pid);
        pid
    }

    //获取进程的句柄
    pub fn get_process_handle(pid: u32) -> HANDLE {
        let handle = unsafe { OpenProcess(PROCESS_ALL_ACCESS, FALSE, pid) };
        println!("窗口进程句柄为：{:?}", handle);
        handle
    }

    //读取内存4字节数据
    pub fn r4(&self, base_adress: i32) -> i32 {
        let mut buffer = 0i32;
        if unsafe {
            ReadProcessMemory(
                self.process_handle,
                base_adress as *const c_void,
                &mut buffer as *mut c_int as *mut c_void,
                size_of::<i32>(),
                &mut 0,
            )
        } == FALSE
        {
            //读取内存失败
            println!("Error reading memory:{:?}", io::Error::last_os_error());
        }
        buffer
    }

    //读取内存数据
    pub fn read_memory(&self, handle: *mut c_void, base_adress: i32, buffer: &mut f32) -> bool {
        if unsafe {
            ReadProcessMemory(
                handle,
                base_adress as *const c_void,
                buffer as *mut c_float as *mut c_void,
                size_of::<f32>(),
                &mut 0,
            )
        } == FALSE
        {
            //读取内存失败
            println!("Error reading memory:{:?}", io::Error::last_os_error());
            false
        } else {
            true
        }
    }

    //写入内存数据
    pub fn write_memory(&self, base_adress: i32, buffer: f32) -> bool {
        println!("write base_adress is : {}", base_adress);
        println!("write buffer is : {}", buffer);
        if unsafe {
            WriteProcessMemory(
                self.process_handle,
                base_adress as *mut c_void,
                &buffer as *const c_float as *const c_void,
                size_of::<f32>(),
                0 as *mut usize,
            )
        } == FALSE
        {
            //写入内存失败
            println!("Error write memory:{:?}", io::Error::last_os_error());
            false
        } else {
            true
        }
    }

    //写入内存数据
    pub fn write_memory_int(&self, base_adress: i32, buffer: i32) -> bool {
        println!("write base_adress is : {}", base_adress);
        println!("write buffer is : {}", buffer);
        if unsafe {
            WriteProcessMemory(
                self.process_handle,
                base_adress as *mut c_void,
                &buffer as *const c_int as *const c_void,
                size_of::<i32>(),
                0 as *mut usize,
            )
        } == FALSE
        {
            //写入内存失败
            println!("Error write memory:{:?}", io::Error::last_os_error());
            false
        } else {
            true
        }
    }
}
