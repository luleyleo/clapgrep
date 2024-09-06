use dotext::{doc::OpenOfficeDoc, *};
use std::{error::Error, io::Read, path::Path};

mod pdf;

pub trait ExtendedTrait {
    fn name(&self) -> String;
    ///lowercase extensions
    fn extensions(&self) -> Vec<String>;
    fn to_string(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>>;
}

#[derive(Clone, Copy, Debug)]
pub enum ExtendedType {
    Pdf,
    Office,
}

impl ExtendedTrait for ExtendedType {
    fn extensions(&self) -> Vec<String> {
        match self {
            ExtendedType::Pdf => vec!["pdf".to_string()],
            ExtendedType::Office => vec![
                "docx".to_string(),
                "xlsx".to_string(),
                "pptx".to_string(),
                "odt".to_string(),
                // "ods".to_string(),
                "odp".to_string(),
            ],
        }
    }

    fn to_string(&self, path: &Path) -> Result<String, Box<dyn std::error::Error>> {
        match self {
            ExtendedType::Pdf => Ok(pdf::extract_pdf(path)?),
            ExtendedType::Office => Ok(extract_office(path)?),
        }
    }

    fn name(&self) -> String {
        match self {
            ExtendedType::Pdf => "Pdf",
            ExtendedType::Office => "Office",
        }
        .to_string()
    }
}

impl From<&str> for ExtendedType {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            "pdf" => ExtendedType::Pdf,
            "office" => ExtendedType::Office,
            _ => panic!("unknown extended type"),
        }
    }
}

fn extract_office(path: &Path) -> Result<String, Box<dyn Error>> {
    let ext = path
        .extension()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();

    let mut string = String::new();
    match ext.as_str() {
        "docx" => {
            let mut docx = Docx::open(path)?;
            docx.read_to_string(&mut string)?;
        }
        "xlsx" => {
            let mut xlsx = Xlsx::open(path)?;
            xlsx.read_to_string(&mut string)?;
        }
        "pptx" => {
            let mut pptx = Pptx::open(path)?;
            pptx.read_to_string(&mut string)?;
        }
        "odt" => {
            let mut odt = Odt::open(path)?;
            odt.read_to_string(&mut string)?;
        }
        // "ods" => {
        //     let ods = Ods::open(&path)?;
        //     ods.read_to_string(&mut string)?;
        // }
        "odp" => {
            let mut odp = Odp::open(path)?;
            odp.read_to_string(&mut string)?;
        }
        _ => return Err("unknown extension".into()),
    }
    Ok(string)
}
