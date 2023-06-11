#[cfg(windows)]
extern crate winapi;

use std::io::Error;
use std::mem::{size_of, MaybeUninit};
use std::{io, ptr::NonNull};
use winapi::ctypes::c_void;
use winapi::shared::minwindef::{DWORD, FALSE, LPVOID};
use winapi::shared::ntdef::{LPCSTR, LUID, NULL};
use winapi::shared::winerror;
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::winbase::{self, LookupPrivilegeValueA};
use winapi::um::winnt::{
    self, MEMORY_BASIC_INFORMATION, PAGE_EXECUTE_READWRITE, SE_PRIVILEGE_ENABLED, TOKEN_PRIVILEGES,
};

use crate::memory_utils::MemoryUtils;
use crate::torchlight_utils::TorchlightUtils;

#[cfg(windows)]
fn print_message(msg: &str) -> Result<i32, Error> {
    use std::ffi::OsStr;
    use std::iter::once;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;
    use winapi::um::winuser::{MessageBoxW, MB_OK};

    let wide: Vec<u16> = OsStr::new(msg).encode_wide().chain(once(0)).collect();
    let ret = unsafe { MessageBoxW(null_mut(), wide.as_ptr(), wide.as_ptr(), MB_OK) };
    if ret == 0 {
        Err(Error::last_os_error())
    } else {
        Ok(ret)
    }
}

#[cfg(not(windows))]
fn print_message(msg: &str) -> Result<(), Error> {
    println!("{}", msg);
    Ok(())
}

//使用EnumProcesses获取所有进程
pub fn enum_proc() -> io::Result<Vec<u32>> {
    let mut size = 0;
    let mut pids = Vec::<DWORD>::with_capacity(2048);
    if unsafe {
        winapi::um::psapi::EnumProcesses(
            pids.as_mut_ptr(),
            (pids.capacity() * std::mem::size_of::<DWORD>()) as u32,
            &mut size,
        )
    } == FALSE
    {
        return Err(io::Error::last_os_error());
    }

    Ok(pids)
}

struct WinapiUtils {
    pub pid: u32,
    pub handle: NonNull<c_void>,
}

impl WinapiUtils {
    //调整debug权限
    //privilege :name of privilege to enable/disable
    //enable_privilege: to enable or disable privilege
    pub fn set_privilege(&self, privilege: LPCSTR, enable_privilege: bool) -> Result<bool, Error> {
        //权限在本地的唯一标识
        let mut luid = LUID {
            LowPart: 20,
            HighPart: 0,
        };

        // if unsafe {
        //     // lookup privilege on local
        //     // privilege to lookup
        //     // receives LUID of privilege
        //     //winbase::LookupPrivilegeNameW(lpSystemName, lpLuid, lpName, cchName)
        //     winbase::LookupPrivilegeValueA(
        //         &mut 0,
        //         privilege,
        //         &mut luid,
        //     )
        // } == FALSE
        // {
        //     let error = Error::last_os_error();
        //     println!("LookupPrivilegeValue error:{}", error);
        //     return Err(error);
        // }
        let mut tp = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [winnt::LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: 0,
            }],
        };

        if enable_privilege {
            //SE_PRIVILEGE_ENABLED表示打开权限
            tp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
        } else {
            tp.Privileges[0].Attributes = 0;
        }

        // Enable the privilege or disable all privileges.

        if unsafe {
            AdjustTokenPrivileges(
                self.handle.as_ptr(),
                FALSE,
                &mut tp,
                size_of::<TOKEN_PRIVILEGES>() as u32,
                &mut tp,
                &mut 0,
            )
        } == FALSE
        {
            let error = Error::last_os_error();
            println!("AdjustTokenPrivileges error: {}", error);
            return Err(error);
        }

        if Error::last_os_error().raw_os_error().unwrap() as u32 == winerror::ERROR_NOT_ALL_ASSIGNED
        {
            println!("The token does not have the specified privilege. \n");
            return Ok(false);
        }
        Ok(true)
    }

    //打开进程
    pub fn open_process(pid: u32) -> io::Result<Self> {
        NonNull::new(unsafe {
            winapi::um::processthreadsapi::OpenProcess(
                winnt::PROCESS_QUERY_INFORMATION
                    | winnt::PROCESS_VM_READ
                    | winnt::PROCESS_VM_WRITE
                    | winnt::PROCESS_VM_OPERATION
                    | winnt::PROCESS_ALL_ACCESS,
                FALSE,
                pid,
            )
        })
        .map(|handle| WinapiUtils { pid, handle })
        .ok_or_else(io::Error::last_os_error)
    }

    //获取内存区域
    pub fn memory_regions(&self) -> Vec<MEMORY_BASIC_INFORMATION> {
        let mut base = 0;
        let mut regions = vec![];
        let mut info = MaybeUninit::uninit();

        loop {
            let written = unsafe {
                winapi::um::memoryapi::VirtualQueryEx(
                    self.handle.as_ptr(),
                    base as *const _,
                    info.as_mut_ptr(),
                    size_of::<MEMORY_BASIC_INFORMATION>(),
                )
            };
            if written == 0 {
                break;
            }

            let info = unsafe { info.assume_init() };
            //计算下一块区域base
            base = info.BaseAddress as usize + info.RegionSize;
            //保存
            regions.push(info);
        }

        let mask = winnt::PAGE_EXECUTE_READWRITE
            | winnt::PAGE_EXECUTE_WRITECOPY
            | winnt::PAGE_READWRITE
            | winnt::PAGE_WRITECOPY;
        //过滤掉系统模块，无效区域

        return regions
            .into_iter()
            .filter(|x| !(x.BaseAddress as u32 > 0x70000000 && (x.BaseAddress as u32) < 0x80000000))
            .filter(|p| (p.Protect & mask) != 0)
            .collect();
    }

    //读取内存数据
    pub fn read_memory(&self, address: usize) -> Result<f32, Error> {
        let mut value_read = Vec::<f32>::with_capacity(4 * 1024 * 8);
        let mut old_protect: u32 = 0;
        //已经读取到的字节数
        let mut readed_nums = 0usize;

        if unsafe {
            //使用VirtualProtectEx解除内存保护
            // VirtualProtectEx根据以下内容输入即可使用。
            // BOOL VirtualProtectEx(
            // HANDLE hProcess, // 要修改内存的进程句柄
            // LPVOID lpAddress, // 要修改内存的起始地址
            // DWORD dwSize, // 页区域大小
            // DWORD flNewProtect, // 新访问方式
            // PDWORD lpflOldProtect // 原访问方式 用于保存改变前的保护属性 易语言要传址

            winapi::um::memoryapi::VirtualProtectEx(
                self.handle.as_ptr(),
                address as LPVOID,
                4 * 1024 * 8,
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
        } == FALSE
        {
            let error = io::Error::last_os_error();
            println!("{}", error);
            return Err(error);
        }

        if unsafe {
            winapi::um::memoryapi::ReadProcessMemory(
                self.handle.as_ptr(),
                address as LPVOID,
                value_read.as_mut_ptr().cast(),
                4 * 1024 * 8,
                &mut readed_nums,
            )
        } == FALSE
        {
            let error = io::Error::last_os_error();
            println!("{}", error);
            return Err(error);
        }
        println!("已经读取到字节数：{}个", readed_nums);
        println!("{:#?}", value_read);
        Ok(0.0f32)
    }

    //写入内存数据
    pub fn write_memory(&self, address: usize, value: &mut [usize]) {
        let mut write = 0;
        let mut old_protect: u32 = 0;
        if unsafe {
            //使用VirtualProtectEx解除内存保护
            winapi::um::memoryapi::VirtualProtectEx(
                self.handle.as_ptr(),
                address as LPVOID,
                value.len(),
                PAGE_EXECUTE_READWRITE,
                &mut old_protect,
            )
        } == FALSE
        {
            return;
        }

        if unsafe {
            //写入特定数据
            winapi::um::memoryapi::WriteProcessMemory(
                self.handle.as_ptr(),
                address as LPVOID,
                value.as_ptr().cast(),
                value.len(),
                &mut write,
            )
        } == FALSE
        {
            return;
        }
    }
}

mod memory_utils;
mod torchlight_utils;

fn main() {
    let memory_utils = MemoryUtils::new("OgreD3D9Wnd", "Torchlight II v.1.13.5.12");

    let torchlight_utils = TorchlightUtils::new(memory_utils);

    loop {
        //修改血量为200
        torchlight_utils.hp_infinite(200.0f32);
        //修改蓝量为40
        torchlight_utils.mp_infinite(40.0);
        //修改金币为11111
        torchlight_utils.gold_infinite(11111);
    }
}
