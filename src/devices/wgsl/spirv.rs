use core::{mem::size_of_val, str::FromStr};

use naga::{
    back::spv::{Options, PipelineOptions},
    valid::{ModuleInfo, ValidationError},
    WithSpan,
};

pub struct Spirv {
    words: Vec<u32>,
}

impl Spirv {
    pub fn from_wgsl(src: impl AsRef<str>) -> Result<Self, TranslateError> {
        let (module, info) = parse_and_validate_src(src.as_ref())?;
        let words = write_spirv(&module, &info)?;
        Ok(Spirv { words })
    }

    #[inline]
    pub fn as_byte_slice(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.words.as_ptr() as *const u8,
                size_of_val(self.words.as_slice()),
            )
        }
    }
}

pub fn parse_and_validate_src(src: &str) -> Result<(naga::Module, ModuleInfo), TranslateError> {
    let mut frontend = naga::front::wgsl::Frontend::new();

    let module = frontend.parse(src).map_err(TranslateError::Frontend)?;

    let mut validator = naga::valid::Validator::new(
        naga::valid::ValidationFlags::all(),
        naga::valid::Capabilities::all(),
    );

    let info = validator
        .validate(&module)
        .map_err(TranslateError::Validate)?;
    Ok((module, info))
}

pub fn write_spirv(module: &naga::Module, info: &ModuleInfo) -> Result<Vec<u32>, TranslateError> {
    let mut words = Vec::new();

    let mut writer =
        naga::back::spv::Writer::new(&Options::default()).map_err(TranslateError::Backend)?;
    writer
        .write(
            &module,
            &info,
            Some(&PipelineOptions {
                shader_stage: naga::ShaderStage::Compute,
                entry_point: "main".into(),
            }),
            &None,
            &mut words,
        )
        .map_err(TranslateError::Backend)?;

    Ok(words)
}

pub enum TranslateError {
    Validate(WithSpan<ValidationError>),
    Frontend(naga::front::wgsl::ParseError),
    Backend(naga::back::spv::Error),
}

impl FromStr for Spirv {
    type Err = TranslateError;

    #[inline]
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Spirv::from_wgsl(s)
    }
}
