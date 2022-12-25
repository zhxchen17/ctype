use std::fmt;
use std::sync::Mutex;

use once_cell::sync::Lazy;

static ERRORS: Lazy<Mutex<Vec<String>>> = Lazy::new(|| Mutex::new(vec![]));

pub enum Error {
    UndefinedName(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let (code, msg) = match self {
            Error::UndefinedName(name) => (1, format!("Undefined name: {}", name)),
        };
        write!(f, "E{:04} {}", code, msg)
    }
}

pub fn report_error(error: Error) {
    let mut errors = ERRORS.lock().unwrap();
    errors.push(format!("{}", error));
}

pub fn dump_errors() {
    let errors = ERRORS.lock().unwrap();
    errors.iter().for_each(|e| println!("{}", e));
}
