use base64::prelude::*;
use plugin_shared::PluginCollectData;
use std::alloc::{alloc, dealloc, Layout};
use std::ptr::null_mut;

// TODO: We need to do some benchmarking ... if its worth it to use this library (which seems not to have been updated in the last 3 years)
// #[global_allocator]
// static ALLOCATOR: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[no_mangle]
pub extern "C" fn collect(port: u32) -> *mut PluginCollectData {
    let raw_str = req_data(port);
    let data_base64_encoded = BASE64_STANDARD.encode(&raw_str);
    let data_len = data_base64_encoded.len() as i32;


    // Rust's String type automatically manages memory, which differs from languages like C
    // that require explicit memory management. So we can interface with such systems and have precise
    // control over memory layout, manual memory management is required.

    // First we determine the amount of memory needed to store the encoded string plus a null terminator.
    let data_layout = Layout::array::<u8>(data_len as usize + 1).unwrap();
    let data_offset = unsafe {
        // We allocate a block of memory according to the specified layout.
        // `alloc` returns a raw pointer to the beginning of the block.
        let data_alloc_ptr = alloc(data_layout) as *mut u8;
        if data_alloc_ptr.is_null() {
            // Allocation failed: we return a null pointer to indicate the error.
            return null_mut();
        }

        // Safely copy the encoded string into the allocated memory.
        std::ptr::copy_nonoverlapping(data_base64_encoded.as_ptr(), data_alloc_ptr, data_len as usize );

        // Lastly, we append a null terminator to the end of the string to comply with C-string
        // conventions, which expect strings to be null-terminated.
        *data_alloc_ptr.add(data_len as usize ) = 0;
        data_alloc_ptr as i32
    };

    // Allocate memory for the PluginCollectData struct
    // with the same reasons as with string above.
    let data_dto_layout = Layout::new::<PluginCollectData>();
    let data_dto_ptr = unsafe {
        let data_dto_alloc_ptr = alloc(data_dto_layout) as *mut PluginCollectData;
        if data_dto_alloc_ptr.is_null() {
            dealloc(data_offset as *mut u8, data_layout);
            return null_mut();
        }
        (*data_dto_alloc_ptr).offset = data_offset;
        (*data_dto_alloc_ptr).len = data_len;

        data_dto_alloc_ptr
    };

    // Printing pointer values (effectively the offsets from the base of the allocated memory)
    println!("[PLUGIN]: Buffer DTO struct created at: {:p}", data_dto_ptr);
    println!("[PLUGIN]: Data offset: {:?}", data_offset);

    unsafe {
        let data_content = std::str::from_utf8_unchecked(std::slice::from_raw_parts(data_offset as *mut u8, data_len as usize));
        println!("[PLUGIN]: Data at offset: {}", data_content);
        println!("[PLUGIN]: Buffer DTO struct details: {:?}", *data_dto_ptr);
    }

    println!("[PLUGIN]: Returning this pointer: {:p}", data_dto_ptr);
    data_dto_ptr
}

// This function just acts as a dummy
fn req_data(port: u32) -> String {
    let s = format!(
        "some string information 123 with - data {{inside}} and port: {}",
        port
    );
    s
}
