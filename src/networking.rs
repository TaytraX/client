use std::ffi::CString;
use std::thread;
use std::time::Duration;

type HANDLE = *mut std::ffi::c_void;
type LPVOID = *mut std::ffi::c_void;

#[link(name = "kernel32")]
unsafe extern "system" {
    fn OpenFileMappingA(
        dwDesiredAccess: u32,
        bInheritHandle: i32,
        lpName: *const i8,
    ) -> HANDLE;

    fn MapViewOfFile(
        hFileMappingObject: HANDLE,
        dwDesiredAccess: u32,
        dwFileOffsetHigh: u32,
        dwFileOffsetLow: u32,
        dwNumberOfBytesToMap: usize,
    ) -> LPVOID;
}

const FILE_MAP_READ: u32 = 0x0004;

pub fn get_chunk() -> Box<[f64; 32768]> {
    let handle = loop {
        let h = unsafe { OpenFileMappingA(FILE_MAP_READ, 0, CString::new("SCENE").unwrap().as_ptr()) };

        if !h.is_null() {
            break h;
        }

        println!("Waiting for shared memory...");
        thread::sleep(Duration::from_millis(500));
    };

    let ptr = unsafe { MapViewOfFile(handle, FILE_MAP_READ, 0, 0, 32768 * size_of::<f64>()) };

    let slice = unsafe {
        std::slice::from_raw_parts(ptr as *const f64, 32768)
    };

    Box::new(slice.try_into().expect("slice fits into f64"))
}