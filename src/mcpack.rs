//! Bedrock .mcpack archive reader and resource extractor.

use std::fs::File;
use std::io::{BufReader, Read, Seek};
use std::path::Path;
use zip::ZipArchive;

use crate::anim::BedrockAnimationFile;

/// Errors that can occur when reading an mcpack archive or extracting assets.
#[derive(Debug)]
pub enum McpackError {
    Io(std::io::Error),
    Zip(zip::result::ZipError),
    Json(serde_json::Error),
    FileNotFound(String),
}

impl std::fmt::Display for McpackError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            McpackError::Io(e) => write!(f, "I/O error: {}", e),
            McpackError::Zip(e) => write!(f, "ZIP error: {}", e),
            McpackError::Json(e) => write!(f, "JSON error: {}", e),
            McpackError::FileNotFound(name) => write!(f, "File '{}' not found in mcpack", name),
        }
    }
}

impl std::error::Error for McpackError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            McpackError::Io(e) => Some(e),
            McpackError::Zip(e) => Some(e),
            McpackError::Json(e) => Some(e),
            McpackError::FileNotFound(_) => None,
        }
    }
}

impl From<std::io::Error> for McpackError {
    fn from(e: std::io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<zip::result::ZipError> for McpackError {
    fn from(e: zip::result::ZipError) -> Self {
        match e {
            zip::result::ZipError::FileNotFound => McpackError::FileNotFound("<unknown>".into()),
            other => Self::Zip(other),
        }
    }
}

impl From<serde_json::Error> for McpackError {
    fn from(e: serde_json::Error) -> Self {
        Self::Json(e)
    }
}

/// High-performance Bedrock `.mcpack` zip archive reader.
pub struct McpackArchive<R> {
    archive: ZipArchive<R>,
}

impl McpackArchive<BufReader<File>> {
    /// Opens an mcpack file from the filesystem using a buffered reader.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, McpackError> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        Self::from_reader(reader)
    }
}

impl<R: Read + Seek> McpackArchive<R> {
    /// Constructs an `McpackArchive` from an arbitrary `Read + Seek` stream.
    pub fn from_reader(reader: R) -> Result<Self, McpackError> {
        let archive = ZipArchive::new(reader)?;
        Ok(Self { archive })
    }

    /// Total number of files in the archive.
    #[inline]
    pub fn len(&self) -> usize {
        self.archive.len()
    }

    /// Returns true if the archive contains no files.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.archive.is_empty()
    }

    /// Returns an iterator over all file names in the archive.
    pub fn file_names(&self) -> impl Iterator<Item = &str> {
        self.archive.file_names()
    }

    /// Checks whether a file exists in the archive.
    pub fn has_file(&self, name: &str) -> bool {
        self.archive.file_names().any(|f| f == name)
    }

    /// Checks whether a file exists in the archive (alias of `has_file`).
    #[inline]
    pub fn contains_file(&self, name: &str) -> bool {
        self.has_file(name)
    }

    /// Reads an entire file into a UTF-8 String.
    pub fn read_file_to_string(&mut self, name: &str) -> Result<String, McpackError> {
        let mut file = self.archive.by_name(name).map_err(|e| match e {
            zip::result::ZipError::FileNotFound => McpackError::FileNotFound(name.to_string()),
            other => McpackError::Zip(other),
        })?;
        let mut content = String::with_capacity(file.size() as usize);
        file.read_to_string(&mut content)?;
        Ok(content)
    }

    /// Reads an entire file into a UTF-8 String (convenience alias).
    #[inline]
    pub fn read_to_string(&mut self, name: &str) -> Result<String, McpackError> {
        self.read_file_to_string(name)
    }

    /// Reads an entire file into a byte vector.
    pub fn read_file_to_bytes(&mut self, name: &str) -> Result<Vec<u8>, McpackError> {
        let mut file = self.archive.by_name(name).map_err(|e| match e {
            zip::result::ZipError::FileNotFound => McpackError::FileNotFound(name.to_string()),
            other => McpackError::Zip(other),
        })?;
        let mut content = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut content)?;
        Ok(content)
    }

    /// Reads an entire file into a byte vector (convenience alias).
    #[inline]
    pub fn read_file(&mut self, name: &str) -> Result<Vec<u8>, McpackError> {
        self.read_file_to_bytes(name)
    }

    /// Streams and deserializes a JSON file directly into type `T`.
    pub fn read_json<T: serde::de::DeserializeOwned>(&mut self, name: &str) -> Result<T, McpackError> {
        let mut file = self.archive.by_name(name).map_err(|e| match e {
            zip::result::ZipError::FileNotFound => McpackError::FileNotFound(name.to_string()),
            other => McpackError::Zip(other),
        })?;
        let result = serde_json::from_reader(&mut file)?;
        Ok(result)
    }

    /// Deserializes a Bedrock animation file from the archive.
    pub fn load_animation_file(&mut self, name: &str) -> Result<BedrockAnimationFile, McpackError> {
        self.read_json(name)
    }

    /// Lists all animation JSON files in the pack (`animations/*.json` and subpacks).
    pub fn list_animation_files(&self) -> Vec<String> {
        self.archive
            .file_names()
            .filter(|n| n.contains("animations/") && n.ends_with(".json"))
            .map(|s| s.to_string())
            .collect()
    }

    /// Lists all attachable JSON files in the pack (`attachables/*.json` and subpacks).
    pub fn list_attachable_files(&self) -> Vec<String> {
        self.archive
            .file_names()
            .filter(|n| n.contains("attachables/") && n.ends_with(".json"))
            .map(|s| s.to_string())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Cursor, Write};
    use zip::write::SimpleFileOptions;
    use zip::ZipWriter;

    fn create_test_zip() -> Vec<u8> {
        let mut buffer = Vec::new();
        {
            let mut zip = ZipWriter::new(Cursor::new(&mut buffer));
            let options = SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);

            zip.start_file("manifest.json", options).unwrap();
            zip.write_all(b"{\"format_version\":3,\"header\":{\"name\":\"test_pack\"}}").unwrap();

            zip.start_file("animations/test.json", options).unwrap();
            zip.write_all(b"{\"format_version\":\"1.8.0\",\"animations\":{\"anim.test\":{\"loop\":true}}}")
                .unwrap();

            zip.finish().unwrap();
        }
        buffer
    }

    #[test]
    fn test_mcpack_in_memory_reading() {
        let bytes = create_test_zip();
        let mut archive = McpackArchive::from_reader(Cursor::new(bytes)).expect("Failed to open archive");

        assert_eq!(archive.len(), 2);
        assert!(!archive.is_empty());
        assert!(archive.has_file("manifest.json"));
        assert!(archive.has_file("animations/test.json"));
        assert!(!archive.has_file("non_existent.json"));

        let manifest_str = archive.read_to_string("manifest.json").expect("Read manifest");
        assert!(manifest_str.contains("test_pack"));

        let manifest_bytes = archive.read_file("manifest.json").expect("Read bytes");
        assert_eq!(manifest_bytes, manifest_str.as_bytes());
    }

    #[test]
    fn test_mcpack_load_animation_file() {
        let bytes = create_test_zip();
        let mut archive = McpackArchive::from_reader(Cursor::new(bytes)).expect("Failed to open archive");

        let anim_file = archive.load_animation_file("animations/test.json").expect("Load anim");
        assert_eq!(anim_file.format_version, "1.8.0");
        assert!(anim_file.animations.contains_key("anim.test"));

        let anim_files = archive.list_animation_files();
        assert_eq!(anim_files, vec!["animations/test.json"]);
    }

    #[test]
    fn test_mcpack_missing_file_returns_err() {
        let bytes = create_test_zip();
        let mut archive = McpackArchive::from_reader(Cursor::new(bytes)).unwrap();
        let err = archive.read_file("does_not_exist.json");
        assert!(err.is_err(), "Missing file must return Err, not panic");
        match err {
            Err(McpackError::FileNotFound(name)) => assert_eq!(name, "does_not_exist.json"),
            other => panic!("Expected FileNotFound, got {:?}", other),
        }
    }

    #[test]
    fn test_mcpack_corrupt_bytes_returns_err() {
        let corrupt = b"not a zip file at all";
        let res = McpackArchive::from_reader(Cursor::new(corrupt));
        assert!(res.is_err(), "Corrupt zip bytes must return Err without panicking");
    }
}
