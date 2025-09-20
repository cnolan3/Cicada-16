use thiserror::Error;

#[derive(Error, Debug, PartialEq)]
pub enum AssemblyError {
    #[error("Syntax Error: {0}")]
    PestError(#[from] pest::error::Error<crate::parser::Rule>),

    #[error("Structural Error on line {line}: {reason}")]
    StructuralError { line: usize, reason: String },

    #[error("Semantic Error on line {line}: {reason}")]
    SemanticError { line: usize, reason: String },

    #[error("Semantic Error: {reason}")]
    SemanticErrorNoLine { reason: String },
}
