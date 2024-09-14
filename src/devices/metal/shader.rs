use metal::*;

pub struct MetalShader {
    pub function: Function,
}

impl MetalShader {
    pub fn new(device: &Device, source: &str, function_name: &str) -> MetalShader {
        let library = device.new_library_with_source(source, &CompileOptions::new()).unwrap();
        let function = library.get_function(function_name, None).unwrap();
        MetalShader {
            function,
        }
    }
}
