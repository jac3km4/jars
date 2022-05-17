use std::ffi::OsStr;
use std::io::{Read, Seek};
use std::path::Path;

use cafebabe::{parse_class, parse_class_with_options, ClassFile, ParseOptions};
use zip::read::ZipFile;

use crate::result::{Error, Result};

/// A JAR archive containing Java classes.
#[derive(Debug)]
pub struct Jar<R> {
    zip: zip::ZipArchive<R>,
}

impl<R: Read + Seek> Jar<R> {
    pub fn new(source: R) -> Result<Self> {
        let zip = zip::ZipArchive::new(source)?;
        Ok(Self { zip })
    }

    /// Returns an iterator over all classes in the archive, each represented as a [`JarEntry`].
    pub fn classes(&mut self) -> ClassIter<R> {
        ClassIter {
            zip: &mut self.zip,
            index: 0,
        }
    }
}

#[derive(Debug)]
pub struct JarEntry(Box<[u8]>);

impl JarEntry {
    /// Attempts to parse this entry as a [`ClassFile`].
    #[inline]
    pub fn parse(&self) -> Result<ClassFile> {
        parse_class(&self.0).map_err(Error::ClassError)
    }

    /// Attempts to parse this entry as a [`ClassFile`], ignoring the bytecode of it's methods.
    #[inline]
    pub fn parse_without_bytecode(&self) -> Result<ClassFile> {
        parse_class_with_options(&self.0, ParseOptions::default().parse_bytecode(false))
            .map_err(Error::ClassError)
    }
}

pub struct ClassIter<'a, R> {
    zip: &'a mut zip::ZipArchive<R>,
    index: usize,
}

impl<'a, R: Read + Seek> Iterator for ClassIter<'a, R> {
    type Item = Result<JarEntry>;

    fn next(&mut self) -> Option<Self::Item> {
        let entry = loop {
            let entry = self.zip.by_index(self.index).ok()?;
            self.index += 1;
            let path: &Path = entry.name().as_ref();
            if path.extension() == Some(OsStr::new("class")) {
                break entry;
            }
        };
        Some(read_class(entry))
    }
}

fn read_class(mut file: ZipFile) -> Result<JarEntry> {
    let mut buffer = vec![0; file.size() as usize];
    file.read_exact(&mut buffer)?;
    Ok(JarEntry(buffer.into_boxed_slice()))
}
