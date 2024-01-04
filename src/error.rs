use crate::interpreter::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoliumError<'a> {
    UnknownType { offending_token: &'a str },
    UseOfContentTypeName { word: &'a str },
    ExpectedToken { expected: Token<'a>, got: Token<'a> },
    ExpectedReason { expected: &'a str, got: Token<'a> },
    UnexpectedFileEndWithToken { expected: Token<'a> },
    UnexpectedFileEndWithReason { expected: &'a str },
    DuplicateStyleDefinition,
}

impl<'a> std::fmt::Display for FoliumError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoliumError::UnknownType { offending_token } => write!(f, "Expected content type but got token {offending_token} instead."),
            FoliumError::UseOfContentTypeName { word } => write!(f, "Erroneous usage of {word}, which is the name of a content type, in a disallowed context."),
            FoliumError::ExpectedToken { expected, got } => write!(f, "Expected {expected:?}, got {got:?}."),
            FoliumError::ExpectedReason { expected, got } => write!(f, "Expected {expected}, got {got:?}."),
            FoliumError::UnexpectedFileEndWithToken { expected } => write!(f, "Expected {expected:?} but the file ended abruptly."),
            FoliumError::UnexpectedFileEndWithReason { expected } => write!(f, "Expected {expected:?} but the file ended abruptly."),
            FoliumError::DuplicateStyleDefinition => write!(f, "Duplicate style definition.")
        }
    }
}