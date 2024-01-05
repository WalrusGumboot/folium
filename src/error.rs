use crate::interpreter::{Token, TokenLocation};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FoliumError<'a> {
    UnknownType {
        location: TokenLocation,
        offending_token: &'a str,
    },
    UseOfContentTypeName {
        location: TokenLocation,
        word: &'a str,
    },
    ExpectedToken {
        location: TokenLocation,
        expected: Token<'a>,
        got: Token<'a>,
    },
    ExpectedReason {
        location: TokenLocation,
        expected: &'a str,
        got: Token<'a>,
    },
    UnexpectedFileEndWithToken {
        location: TokenLocation,
        expected: Token<'a>,
    },
    UnexpectedFileEndWithReason {
        location: TokenLocation,
        expected: &'a str,
    },
}

impl<'a> std::fmt::Display for FoliumError<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FoliumError::UnknownType { location, offending_token } => write!(f, "at {location}: Expected content type but got token {offending_token} instead."),
            FoliumError::UseOfContentTypeName { location, word } => write!(f, "at {location}: Erroneous usage of {word}, which is the name of a content type, in a disallowed context."),
            FoliumError::ExpectedToken { location, expected, got } => write!(f, "at {location}: Expected {expected:?}, got {got:?}."),
            FoliumError::ExpectedReason { location, expected, got } => write!(f, "at {location}: Expected {expected}, got {got:?}."),
            FoliumError::UnexpectedFileEndWithToken { location, expected } => write!(f, "at {location}: Expected {expected:?} but the file ended abruptly."),
            FoliumError::UnexpectedFileEndWithReason { location, expected } => write!(f, "at {location}: Expected {expected:?} but the file ended abruptly."),
        }
    }
}
