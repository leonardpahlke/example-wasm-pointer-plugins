use anyhow::{Result, anyhow};
use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use plugin_shared::PluginCollectData;
use wasi_common::{sync::WasiCtxBuilder, WasiCtx};
use wasmtime::{Engine as WasmTimeEngine, *};
use std::alloc::{dealloc, Layout};

fn main() -> Result<()> {
    let (mut store, instance, memory) = setup_wasmtime()?;

    let plugin_port = 8080;
    let collect = instance.get_typed_func::<i32, i32>(&mut store, "collect")?;
    // let deallocate = instance.get_typed_func::<i32, ()>(&mut store, "deallocate")?;

    let data_offset = collect.call(&mut store, plugin_port as i32)?;

    println!("[RUNNER]: Data offset from WebAssembly: {:?}", data_offset);

    if data_offset == 0 || (data_offset as u32) >= memory.data_size(&store).try_into().unwrap() {
        return Err(anyhow!("[RUNNER]: Invalid data offset"));
    }

    let data_ptr = convert_wasm_pointer(data_offset, &memory, &mut store)?;
    let data_struct = unsafe {
        &*(data_ptr as *const PluginCollectData)
    };

    println!("[RUNNER]: Received DTO Data: {:?}", data_struct);
    println!("[RUNNER]: Data offset: {}", data_struct.offset);

    // Access the base64 encoded data in memory
    let encoded_data_ptr = unsafe {
        let raw_memory = memory.data(&store);
        std::slice::from_raw_parts(raw_memory.as_ptr().add(data_struct.offset as usize), data_struct.len as usize)
    };

    // Print the base64 encoded string
    let encoded_str = std::str::from_utf8(encoded_data_ptr)?;
    println!("[RUNNER]: Encoded data (Base64): {}", encoded_str);

    // Decode the base64 encoded string
    let decoded_bytes = BASE64_STANDARD.decode(encoded_str)?;
    let decoded_str = String::from_utf8(decoded_bytes)?;

    // Print the decoded string
    println!("[RUNNER]: Decoded string: {}", decoded_str);

    // Assuming `PluginCollectData` structure includes a mechanism to know when it's safe to deallocate
    // For example, after processing the data:
    // if let Ok(_) = deallocate.call(&mut store, (data_struct.ptr as i32, data_struct.len as i32)) {
    //     println!("Memory deallocated successfully.");
    // } else {
    //     eprintln!("Failed to deallocate memory.");
    // }

    Ok(())
}

fn setup_wasmtime() -> Result<(Store<WasiCtx>, Instance, Memory), anyhow::Error> {
    let engine = WasmTimeEngine::new(Config::new().debug_info(true))?;
    let mut linker = Linker::new(&engine);
    wasi_common::sync::add_to_linker(&mut linker, |s| s)?;

    let wasi = WasiCtxBuilder::new().inherit_stdio().inherit_env()?.build();
    let mut store = Store::new(&engine, wasi);

    let module = Module::from_file(
        &engine,
        "wasm-plugin/target/wasm32-wasi/release/plugin.wasm",
    )?;
    linker.module(&mut store, "", &module)?;
    let instance = linker.instantiate(&mut store, &module)?;
    let memory = instance
        .get_memory(&mut store, "memory")
        .ok_or(anyhow!("[RUNNER]: Failed to find memory"))?;

    Ok((store, instance, memory))
}

/// Converts a WebAssembly memory offset to a host system pointer.
///
/// This function interprets a 32-bit offset from WebAssembly as a pointer in the host's memory space,
/// ensuring that the pointer remains valid within the constraints of the allocated WebAssembly memory.
///
/// # Parameters
/// - `offset`: The 32-bit offset provided by WebAssembly. It is treated explicitly as `u32` to avoid
///   issues with platform-specific pointer sizes.
/// - `memory`: A reference to the WebAssembly memory object which provides access to the WebAssembly's
///   linear memory.
/// - `store`: The context in which this memory exists, needed to access the memory correctly.
///
/// # Returns
/// A `Result` which is either:
/// - A valid host memory pointer (`*const u8`) if the offset is within bounds,
/// - An `Err` containing an `anyhow::Error` describing the issue (e.g., null pointer, offset out of bounds).
///
/// # Errors
/// The function returns an error if:
/// - The offset is zero (interpreted as a null pointer in WebAssembly),
/// - The offset exceeds the size of the allocated memory (out of bounds).
///
/// # Safety
/// The function performs unsafe operations to convert the WebAssembly offset to a pointer.
/// The caller must ensure that the memory accessed through this pointer does not violate
/// Rust's safety guarantees, such as aliasing rules and mutable immutability guarantees.
fn convert_wasm_pointer(offset: i32, memory: &Memory, store: &mut Store<WasiCtx>) -> Result<*const u8, anyhow::Error> {
    let offset = offset as u32;
    let memory_size = memory.data_size(&store) as u32;

    if offset == 0 {
        return Err(anyhow!("[RUNNER]: Null pointer from WebAssembly"));
    }

    if offset > memory_size {
        return Err(anyhow!("[RUNNER]: Pointer offset out of bounds"));
    }

    let pointer = unsafe { memory.data_ptr(&store).add(offset as usize) as *const u8 };

    Ok(pointer)
}

// unsafe fn deallocate(offset: i32, memory: &Memory, store: &mut Store<WasiCtx>) -> Result<(), anyhow::Error> {
//     println!("[WASM-PLUGIN] Deallocate Address Space");

//     // Convert offset to pointer
//     let data_struct = memory.data_ptr(store).add(offset as usize) as *mut PluginCollectData;

//     if data_struct.is_null() {
//         return Err(anyhow!("Null pointer error during deallocation"));
//     }

//     // Access the PluginCollectData struct to get the data pointer and length
//     let data_len = (*data_struct).len;
//     let data_offset = (*data_struct).offset;

//     // Deallocate the data buffer
//     let data_ptr = memory.data_ptr(store).add(data_offset as usize) as *mut u8;
//     let data_layout = Layout::array::<u8>(data_len as usize + 1).unwrap();
//     dealloc(data_ptr, data_layout);

//     // Deallocate the PluginCollectData struct itself
//     let struct_layout = Layout::new::<PluginCollectData>();
//     dealloc(data_struct as *mut u8, struct_layout);

//     Ok(())
// }



#[no_mangle]
pub unsafe extern "C" fn deallocate(struct_data: *mut PluginCollectData) {
    print!("[PLUGIN]: Deallocate Address Space\n");

    if struct_data.is_null() {
        return;
    }

    // Retrieve the length and data pointer stored in the PluginCollectData struct.
    let data_len = (*struct_data).len;
    let data_offset = (*struct_data).offset;

    // Create a layout for the data buffer using the stored length, plus one for the null terminator.
    let data_layout = Layout::array::<u8>(data_len as usize + 1).unwrap();
    dealloc(data_offset as *mut u8, data_layout);

    // Create a layout for the PluginCollectData struct itself and deallocate it.
    let struct_layout = Layout::new::<PluginCollectData>();
    dealloc(struct_data as *mut u8, struct_layout);
}
