use std::{error::Error, path::Path};

pub mod office;
pub mod pdf;

pub type ExtractFn = fn(path: &Path) -> Result<String, Box<dyn Error>>;
