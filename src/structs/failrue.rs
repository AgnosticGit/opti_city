use std::{error::Error, fmt};

#[derive(Debug)]
pub struct Failure {
    pub reply_to: Option<String>,
    pub message: &'static str,
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "reply_to: {:?} message: {:?}",
            self.reply_to, self.message
        )
    }
}

impl Error for Failure {}
