use dotext::{doc::OpenOfficeDoc, *};
use std::{error::Error, io::Read, path::Path};

pub static EXTENSIONS: &[&str] = &["docx", "pptx", "xlsx", "odt", "odp", "ods"];

pub fn extract(path: &Path) -> Result<String, Box<dyn Error>> {
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
