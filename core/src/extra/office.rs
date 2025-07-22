#![allow(clippy::single_match)]

use crate::search::SearchSink;
use anyhow::{anyhow, Context};
use grep::{regex::RegexMatcher, searcher::Searcher};
use std::{error::Error, fs::File, io::Read, mem, path::Path};
use xml::{events::Event, Reader};
use zip::ZipArchive;

pub static EXTENSIONS: &[&str] = &["docx", "pptx", "xlsx", "odt", "odp", "ods"];

pub type Document = String;
pub type Slides = Vec<String>;

pub fn process(
    searcher: &mut Searcher,
    matcher: &RegexMatcher,
    path: &Path,
    sink: &mut SearchSink,
) -> Result<(), Box<dyn Error>> {
    let ext = path.extension().unwrap_or_default().to_string_lossy();

    match ext.as_ref() {
        "pptx" | "odp" => process_presentation(searcher, matcher, path, sink),
        "docx" | "xlsx" | "odt" | "ods" => process_document(searcher, matcher, path, sink),
        _ => Err("unknown extension".into()),
    }
}

fn process_document(
    searcher: &mut Searcher,
    matcher: &RegexMatcher,
    path: &Path,
    sink: &mut SearchSink,
) -> Result<(), Box<dyn Error>> {
    let ext = path.extension().unwrap_or_default().to_string_lossy();

    let string: Document = match ext.as_ref() {
        "docx" => open_docx(path)?,
        "xlsx" => open_xlsx(path)?,
        "odt" => open_odt(path)?,
        "ods" => open_ods(path)?,
        _ => unreachable!(),
    };

    searcher.search_slice(matcher, string.as_bytes(), sink)
}

fn process_presentation(
    searcher: &mut Searcher,
    matcher: &RegexMatcher,
    path: &Path,
    sink: &mut SearchSink,
) -> Result<(), Box<dyn Error>> {
    let ext = path.extension().unwrap_or_default().to_string_lossy();

    let slides: Slides = match ext.as_ref() {
        "pptx" => open_pptx(path)?,
        "odp" => open_odp(path)?,
        _ => unreachable!(),
    };

    for (i, slide) in slides.iter().enumerate() {
        sink.page = Some(i as u64 + 1);
        searcher.search_slice(matcher, slide.as_bytes(), &mut *sink)?
    }

    Ok(())
}

fn get_pptx_slide_index(file_name: &str) -> anyhow::Result<u64> {
    file_name
        .strip_prefix("ppt/slides/slide")
        .context("failed to strip slide name prefix")?
        .strip_suffix(".xml")
        .context("failed to strip slide name suffix")?
        .parse::<u64>()
        .context("failed to parse slide index")
}

pub fn open_pptx<P: AsRef<Path>>(path: P) -> anyhow::Result<Slides> {
    let file = File::open(path.as_ref())?;
    let mut archive = ZipArchive::new(file)?;

    let mut xml_data = Vec::new();

    for i in 0..archive.len() {
        let mut c_file = archive.by_index(i).unwrap();
        if c_file.name().starts_with("ppt/slides/slide") {
            let slide_idx = get_pptx_slide_index(c_file.name())?;

            let mut buff = String::new();
            c_file.read_to_string(&mut buff)?;
            xml_data.push((slide_idx, buff));
        }
    }

    xml_data.sort_by_key(|(idx, _)| *idx);

    let mut buf = Vec::new();
    let mut slides = Slides::new();

    for (_, slide_xml_data) in xml_data.iter() {
        let mut to_read = false;
        let mut xml_reader = Reader::from_str(slide_xml_data);
        let mut text = String::new();
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().into_inner() {
                    b"a:p" => {
                        to_read = true;
                        text.push('\n');
                    }
                    b"a:t" => {
                        to_read = true;
                    }
                    _ => (),
                },
                Ok(Event::Text(e)) => {
                    if to_read {
                        text.push_str(e.decode().unwrap().as_ref());
                        to_read = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(anyhow!(
                        "Error at position {}: {:?}",
                        xml_reader.buffer_position(),
                        e
                    ));
                }
                _ => (),
            }
        }
        slides.push(text);
    }

    Ok(slides)
}

pub fn open_odp<P: AsRef<Path>>(path: P) -> anyhow::Result<Slides> {
    let file = File::open(path.as_ref())?;
    let mut archive = ZipArchive::new(file)?;

    let mut xml_data = String::new();

    for i in 0..archive.len() {
        let mut c_file = archive.by_index(i).unwrap();
        if c_file.name() == "content.xml" {
            c_file.read_to_string(&mut xml_data)?;
            break;
        }
    }

    let mut xml_reader = Reader::from_str(xml_data.as_ref());

    let mut buf = Vec::new();
    let mut slides = Slides::new();

    if !xml_data.is_empty() {
        let mut to_read = false;
        let mut text = String::new();
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().into_inner() {
                    b"text:p" => {
                        to_read = true;
                        text.push('\n');
                    }
                    b"text:span" => {
                        to_read = true;
                    }
                    _ => (),
                },
                Ok(Event::End(ref e)) => match e.name().into_inner() {
                    b"draw:page" => {
                        slides.push(mem::take(&mut text));
                    }
                    _ => (),
                },
                Ok(Event::Text(e)) => {
                    if to_read {
                        text.push_str(e.decode().unwrap().as_ref());
                        to_read = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(anyhow!(
                        "Error at position {}: {:?}",
                        xml_reader.buffer_position(),
                        e
                    ));
                }
                _ => (),
            }
        }
    }

    Ok(slides)
}

pub fn open_docx<P: AsRef<Path>>(path: P) -> anyhow::Result<Document> {
    let file = File::open(path.as_ref())?;
    let mut archive = ZipArchive::new(file)?;

    let mut xml_data = String::new();

    for i in 0..archive.len() {
        let mut c_file = archive.by_index(i).unwrap();
        if c_file.name() == "word/document.xml" {
            c_file.read_to_string(&mut xml_data)?;
            break;
        }
    }

    let mut xml_reader = Reader::from_str(xml_data.as_ref());

    let mut buf = Vec::new();
    let mut txt = Vec::new();

    if !xml_data.is_empty() {
        let mut to_read = false;
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().into_inner() {
                    b"w:p" => {
                        to_read = true;
                        txt.push("\n".to_string());
                    }
                    b"w:t" => to_read = true,
                    _ => (),
                },
                Ok(Event::Text(e)) => {
                    if to_read {
                        txt.push(e.decode().unwrap().to_string());
                        to_read = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(anyhow!(
                        "Error at position {}: {:?}",
                        xml_reader.buffer_position(),
                        e
                    ));
                }
                _ => (),
            }
        }
    }

    Ok(txt.join(""))
}

pub fn open_xlsx<P: AsRef<Path>>(path: P) -> anyhow::Result<Document> {
    let file = File::open(path.as_ref())?;
    let mut archive = ZipArchive::new(file)?;

    let mut xml_data = String::new();
    //        let xml_data_list = Vec::new();

    for i in 0..archive.len() {
        let mut c_file = archive.by_index(i).unwrap();
        if c_file.name() == "xl/sharedStrings.xml"
            || c_file.name().starts_with("xl/charts/")
            || (c_file.name().starts_with("xl/worksheets") && c_file.name().ends_with(".xml"))
        {
            let mut _buff = String::new();
            c_file.read_to_string(&mut _buff)?;
            xml_data += _buff.as_str();
        }
    }

    let mut buf = Vec::new();
    let mut txt = Vec::new();

    if !xml_data.is_empty() {
        let mut to_read = false;
        let mut xml_reader = Reader::from_str(xml_data.as_ref());
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => match e.name().into_inner() {
                    b"t" => {
                        to_read = true;
                        txt.push("\n".to_string());
                    }
                    b"a:t" => {
                        to_read = true;
                        txt.push("\n".to_string());
                    }
                    _ => (),
                },
                Ok(Event::Text(e)) => {
                    if to_read {
                        let text = e.decode().unwrap();
                        txt.push(text.to_string());
                        to_read = false;
                    }
                }
                Ok(Event::Eof) => break, // exits the loop when reaching end of file
                Err(e) => {
                    return Err(anyhow!(
                        "Error at position {}: {:?}",
                        xml_reader.buffer_position(),
                        e
                    ));
                }
                _ => (),
            }
        }
    }

    Ok(txt.join(""))
}

fn open_od<P: AsRef<Path>>(path: P, content_name: &str, tags: &[&str]) -> anyhow::Result<Document> {
    let file = File::open(path.as_ref())?;
    let mut archive = ZipArchive::new(file)?;

    let mut xml_data = String::new();

    for i in 0..archive.len() {
        let mut c_file = archive.by_index(i).unwrap();
        if c_file.name() == content_name {
            c_file.read_to_string(&mut xml_data)?;
            break;
        }
    }

    let mut xml_reader = Reader::from_str(xml_data.as_ref());

    let mut buf = Vec::new();
    let mut txt = Vec::new();

    if !xml_data.is_empty() {
        let mut to_read = false;
        loop {
            match xml_reader.read_event_into(&mut buf) {
                Ok(Event::Start(ref e)) => {
                    for tag in tags {
                        if e.name().into_inner() == tag.as_bytes() {
                            to_read = true;
                            if e.name().into_inner() == b"text:p" {
                                txt.push("\n".to_string());
                            }
                            break;
                        }
                    }
                }
                Ok(Event::Text(e)) => {
                    if to_read {
                        txt.push(e.decode().unwrap().to_string());
                        to_read = false;
                    }
                }
                Ok(Event::Eof) => break,
                Err(e) => {
                    return Err(anyhow!(
                        "Error at position {}: {:?}",
                        xml_reader.buffer_position(),
                        e
                    ));
                }
                _ => (),
            }
        }
    }

    Ok(txt.join(""))
}

pub fn open_odt<P: AsRef<Path>>(path: P) -> anyhow::Result<Document> {
    open_od(path.as_ref(), "content.xml", &["text:p"])
}

pub fn open_ods<P: AsRef<Path>>(path: P) -> anyhow::Result<Document> {
    open_od(path.as_ref(), "content.xml", &["text:p"])
}
