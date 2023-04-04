use thiserror::Error;

#[allow(dead_code)]
/// this file is used to define errors by ourself.
#[derive(Error, Debug, PartialEq, Clone, Copy)]
/// customerized error types
pub enum Errors {
    /// use thiserror to implement display
    #[error("Recieve Key Duplicate")]
    /// duplicate key error defination
    KeyDuplicate,
    /// message content is wrong
    #[error("Message Content Error")]
    MessageContentError,
    /// type conversion error
    #[error("Type Conversion Error")]
    TypeConversionError,
}
/// an alias for Result<T,Errors>, we can simplify our codes
pub(crate) type Result<T> = std::result::Result<T, Errors>;
