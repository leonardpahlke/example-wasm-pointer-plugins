// This shared library provides common definitions used by both the wasm-plugin and the plugin-runner.
// It ensures consistent data structures and types across different parts of the application, 
// facilitating safe and reliable inter-component communication.

// ---

/// Represents a fixed-size data structure for safely passing variable-sized collected data between 
/// the WASM module and the host environment. 
///
/// In WebAssembly, functions are limited to returning single primitive values. This struct provides 
/// a workaround by encapsulating a pointer to the collected data along with its length. The WASM 
/// module can then return a pointer to an instance of this struct, allowing for the transfer of 
/// variable-sized data via a fixed-size format.
#[repr(C)]
#[derive(Debug)]
pub struct PluginCollectData {
    pub offset: i32,
    pub len: i32,
}
