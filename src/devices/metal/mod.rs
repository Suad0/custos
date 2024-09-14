pub mod context;
pub mod ops;
pub mod shader;
pub mod metal_device;
pub mod metal_arrays;

// Declaring submodules for shader
pub mod shader {
    pub mod command;
    pub mod descriptor;
    pub mod operation;
    pub mod pipeline;
    pub mod shader_argument;
}