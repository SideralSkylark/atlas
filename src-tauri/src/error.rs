use std::{fmt, io};

#[derive(Debug)]
pub enum AppError {
    GitError(git2::Error),
    Io(io::Error),
}

impl From<git2::Error> for AppError {
    fn from(err: git2::Error) -> Self {
        AppError::GitError(err)
    }
}

impl From<io::Error> for AppError {
    fn from(err: io::Error) -> Self {
        AppError::Io(err)
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::GitError(e) => {
                write!(f, "Git Error: {}", e)
            }
            AppError::Io(e) => {
                write!(f, "Io Error: {}", e)
            }
        }
    }
}
