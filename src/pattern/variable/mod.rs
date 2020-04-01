mod eval;
mod parse;

#[derive(Debug, PartialEq)]
pub enum Variable {
    Filename,
    Basename,
    Extension,
    ExtensionWithDot,
    LocalCounter,
    GlobalCounter,
    CaptureGroup(usize),
    Uuid,
}