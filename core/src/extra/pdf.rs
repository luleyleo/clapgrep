use crate::search::SearchSink;
use euclid::vec2;
use grep::{regex::RegexMatcher, searcher::Searcher};
use pdf_extract::{
    encryption::DecryptionError, ConvertToFmt, Document, MediaBox, OutputDev, OutputError,
    Transform,
};
use std::{error::Error, fmt::Write, panic::catch_unwind, path::Path};

pub static EXTENSIONS: &[&str] = &["pdf"];

pub fn process(
    searcher: &mut Searcher,
    matcher: &RegexMatcher,
    path: &Path,
    sink: &mut SearchSink,
) -> Result<(), Box<dyn Error>> {
    let text = extract(path)?;
    searcher.search_slice(matcher, text.as_bytes(), sink)?;
    Ok(())
}

fn extract(path: &Path) -> Result<String, Box<dyn Error>> {
    let path = path.to_owned();
    //because the library panics, we need to catch panics
    let res = catch_unwind(|| extract_text(&path));
    Ok(res.map_err(|_| "Panicked".to_string())??)
}

fn extract_text(path: impl AsRef<Path>) -> Result<String, OutputError> {
    let mut s = String::new();
    {
        let mut output = PlainTextOutput::new(&mut s);
        let doc = Document::load(path)?;
        if doc.is_encrypted() {
            return Err(OutputError::PdfError(pdf_extract::Error::Decryption(
                DecryptionError::IncorrectPassword,
            )));
        }
        pdf_extract::output_doc(&doc, &mut output)?;
    }
    Ok(s)
}

struct PlainTextOutput<W: ConvertToFmt> {
    writer: W::Writer,
    last_end: f64,
    last_y: f64,
    first_char: bool,
    flip_ctm: Transform,
}

impl<W: ConvertToFmt> PlainTextOutput<W> {
    pub fn new(writer: W) -> PlainTextOutput<W> {
        PlainTextOutput {
            writer: writer.convert(),
            last_end: 100000.,
            first_char: false,
            last_y: 0.,
            flip_ctm: Transform::identity(),
        }
    }
}

type ArtBox = (f64, f64, f64, f64);

/* There are some structural hints that PDFs can use to signal word and line endings:
 * however relying on these is not likely to be sufficient. */
impl<W: ConvertToFmt> OutputDev for PlainTextOutput<W> {
    fn begin_page(
        &mut self,
        _page_num: u32,
        media_box: &MediaBox,
        _: Option<ArtBox>,
    ) -> Result<(), OutputError> {
        self.flip_ctm = Transform::row_major(1., 0., 0., -1., 0., media_box.ury - media_box.lly);
        Ok(())
    }

    fn end_page(&mut self) -> Result<(), OutputError> {
        writeln!(self.writer)?;
        Ok(())
    }

    fn output_character(
        &mut self,
        trm: &Transform,
        width: f64,
        _spacing: f64,
        font_size: f64,
        char: &str,
    ) -> Result<(), OutputError> {
        let position = trm.post_transform(&self.flip_ctm);
        let transformed_font_size_vec = trm.transform_vector(vec2(font_size, font_size));
        // get the length of one sized of the square with the same area with a rectangle of size (x, y)
        let transformed_font_size =
            (transformed_font_size_vec.x * transformed_font_size_vec.y).sqrt();
        let (x, y) = (position.m31, position.m32);
        if self.first_char {
            if (y - self.last_y).abs() > transformed_font_size * 1.5 {
                // writeln!(self.writer)?;
                write!(self.writer, " ")?;
            }

            // we've moved to the left and down
            if x < self.last_end && (y - self.last_y).abs() > transformed_font_size * 0.5 {
                // writeln!(self.writer)?;
                write!(self.writer, " ")?;
            }

            if x > self.last_end + transformed_font_size * 0.1 {
                write!(self.writer, " ")?;
            }
        }
        //let norm = unicode_normalization::UnicodeNormalization::nfkc(char);
        write!(self.writer, "{}", char)?;
        self.first_char = false;
        self.last_y = y;
        self.last_end = x + width * transformed_font_size;
        Ok(())
    }

    fn begin_word(&mut self) -> Result<(), OutputError> {
        self.first_char = true;
        Ok(())
    }

    fn end_word(&mut self) -> Result<(), OutputError> {
        Ok(())
    }

    fn end_line(&mut self) -> Result<(), OutputError> {
        //write!(self.file, "\n");
        Ok(())
    }
}
