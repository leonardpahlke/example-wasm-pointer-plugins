# example-wasm-pointer-plugins

This example project illustrates how to start a WebAssembly (WASM) module as a plugin or extension and transfer information from the WASM module to the parent module. Given that the standard API for WASM only allows the transfer of integers, this project utilizes WASM pointers to facilitate the transfer of information.
Steps Involved:

1. **Request Data**: Initiate a request for data from the WASM module.
2. **Get WASM Pointer**: Obtain a pointer from the WASM module.
3. **Transform WASM Pointer to Regular Pointer**: Convert the WASM pointer into a regular pointer that can be used by the parent module.
4. **Read Data Transfer Object (DTO)**: The DTO holds a pointer to the value we want to read, along with the lenght of the value that is referenced (see definition below).

```rust
#[repr(C)]
#[derive(Debug)]
pub struct PluginCollectData {
    pub offset: i32,
    pub len: i32,
}
```
 
5. **Read Information from DTO Pointer**: Extract the information from the DTO pointer.
6. **Deallocate Data in WASM Memory (Depending on usecase)**: Depending on the use case, deallocate the data in WASM memory. (Note: This step is not implemented in the current version.)

### How to run:

1. `make build` to build the three applications   
2. `make run` to run the App
3. `make clean` & `make clean-build` to remove any created artifacts

### References:

This project is inspired by:
- https://docs.wasmtime.dev/examples-rust-wasi.html
- https://github.com/bytecodealliance/wasmtime/blob/main/examples/fib-debug/main.rs
