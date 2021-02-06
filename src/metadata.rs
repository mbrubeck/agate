use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::SystemTime;

static SIDECAR_FILENAME: &str = ".meta";

/// A struct to store a string of metadata for each file retrieved from
/// sidecar files called `.lang`.
///
/// These sidecar file's lines should have the format
/// ```text
/// <filename>:<metadata>
/// ```
/// where `<filename>` is only a filename (not a path) of a file that resides
/// in the same directory and `<metadata>` is the metadata to be stored.
/// Lines that start with optional whitespace and `#` are ignored, as are lines
/// that do not fit the basic format.
/// Both parts are stripped of any leading and/or trailing whitespace.
pub(crate) struct FileOptions {
    /// Stores the paths of the side files and when they were last read.
    /// By comparing this to the last write time, we can know if the file
    /// has changed.
    databases_read: BTreeMap<PathBuf, SystemTime>,
    /// Stores the metadata for each file
    file_meta: BTreeMap<PathBuf, PresetMeta>,
    /// The default value to return
    default: PresetMeta,
}

/// A struct to store the different alternatives that a line in the sidecar
/// file can have.
#[derive(Clone, Debug)]
pub(crate) enum PresetMeta {
    /// A line that starts with a semicolon in the sidecar file, or an
    /// empty line (to overwrite the default language command line flag).
    /// ```text
    /// index.gmi: ;lang=en-GB
    /// ```
    /// The content is interpreted as MIME parameters and are appended to what
    /// agate guesses as the MIME type if the respective file can be found.
    Parameters(String),
    /// A line that is neither a `Parameters` line nor a `FullHeader` line.
    /// ```text
    /// strange.file: text/plain; lang=ee
    /// ```
    /// Agate will send the complete line as the MIME type of the request if
    /// the respective file can be found (i.e. a `20` status code).
    FullMime(String),
    /// A line that starts with a digit between 1 and 6 inclusive followed by
    /// another digit and a space (U+0020). In the categories defined by the
    /// Gemini specification you can pick a defined or non-defined status code.
    /// ```text
    /// gone.gmi: 52 This file is no longer available.
    /// ```
    /// Agate will send this header line, CR, LF, and nothing else. Agate will
    /// not try to access the requested file.
    FullHeader(u8, String),
}

impl FileOptions {
    pub(crate) fn new(default: PresetMeta) -> Self {
        Self {
            databases_read: BTreeMap::new(),
            file_meta: BTreeMap::new(),
            default,
        }
    }

    /// Checks wether the database for the respective directory is still
    /// up to date.
    /// Will only return true if the database should be (re)read, i.e. it will
    /// return false if there is no database file in the specified directory.
    fn check_outdated(&self, db_dir: &PathBuf) -> bool {
        let mut db = db_dir.clone();
        db.push(SIDECAR_FILENAME);
        let db = db.as_path();

        if let Ok(metadata) = db.metadata() {
            if !metadata.is_file() {
                // it exists, but it is a directory
                false
            } else if let (Ok(modified), Some(last_read)) =
                (metadata.modified(), self.databases_read.get(db_dir))
            {
                // check that it was last modified before the read
                // if the times are the same, we might have read the old file
                &modified >= last_read
            } else {
                // either the filesystem does not support last modified
                // metadata, so we have to read it again every time; or the
                // file exists but was not read before, so we have to read it
                true
            }
        } else {
            // the file probably does not exist
            false
        }
    }

    /// (Re)reads a specific sidecar file that resides in the specified
    /// directory. The function takes a directory to minimize path
    /// alterations "on the fly".
    /// This function will allways try to read the file, even if it is current.
    fn read_database(&mut self, db_dir: &PathBuf) {
        log::trace!("reading database for {:?}", db_dir);
        let mut db = db_dir.clone();
        db.push(SIDECAR_FILENAME);
        let db = db.as_path();

        if let Ok(file) = std::fs::File::open(db) {
            let r = BufReader::new(file);
            r.lines()
                // discard any I/O errors
                .filter_map(|line| line.ok())
                // filter out comment lines
                .filter(|line| !line.trim_start().starts_with('#'))
                .for_each(|line| {
                    // split line at colon
                    let parts = line.splitn(2, ':').collect::<Vec<_>>();
                    // only continue if line fits the format
                    if parts.len() == 2 {
                        // generate workspace-unique path
                        let mut path = db_dir.clone();
                        path.push(parts[0].trim());
                        // parse the line
                        let header = parts[1].trim();

                        let preset = if header.is_empty() || header.starts_with(';') {
                            PresetMeta::Parameters(header.to_string())
                        } else if matches!(header.chars().next(), Some('1'..='6')) {
                            if header.len() < 3
                                || !header.chars().nth(1).unwrap().is_ascii_digit()
                                || !header.chars().nth(2).unwrap().is_whitespace()
                            {
                                log::error!("Line for {:?} starts like a full header line, but it is incorrect; ignoring it.", path);
                                return;
                            }
                            let separator = header.chars().nth(2).unwrap();
                            if separator != ' ' {
                                // the Gemini specification says that the third
                                // character has to be a space, so correct any
                                // other whitespace to it (e.g. tabs)
                                log::warn!("Full Header line for {:?} has an invalid character, treating {:?} as a space.", path, separator);
                            }
                            let status = header.chars()
                                .take(2)
                                .collect::<String>()
                                .parse::<u8>()
                                // unwrap since we alread checked it's a number
                                .unwrap();
                            // not taking a slice here because the separator
                            // might be a whitespace wider than a byte
                            let meta = header.chars().skip(3).collect::<String>();
                            PresetMeta::FullHeader(status, meta)
                        } else {
                            // must be a MIME type, but without status code
                            PresetMeta::FullMime(header.to_string())
                        };
                        self.file_meta.insert(path, preset);
                    }
                });
            self.databases_read
                .insert(db_dir.clone(), SystemTime::now());
        }
    }

    /// Get the metadata for the specified file. This might need to (re)load a
    /// single sidecar file.
    /// The file path should consistenly be either absolute or relative to the
    /// working/content directory. If inconsisten file paths are used, this can
    /// lead to loading and storing sidecar files multiple times.
    pub fn get(&mut self, file: &PathBuf) -> PresetMeta {
        let dir = file.parent().expect("no parent directory").to_path_buf();
        if self.check_outdated(&dir) {
            self.read_database(&dir);
        }

        self.file_meta.get(file).unwrap_or(&self.default).clone()
    }
}
