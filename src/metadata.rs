use std::collections::BTreeMap;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::time::SystemTime;

/// A struct to store a string of metadata for each file retrieved from
/// sidecar files called `.lang`.
///
/// These sidecar file's lines should have the format
/// ```text
/// <filename>:<metadata>\n
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
    file_meta: BTreeMap<PathBuf, String>,
    /// The default value to return
    default: String,
}

impl FileOptions {
    pub(crate) fn new(default: &String) -> Self {
        Self {
            databases_read: BTreeMap::new(),
            file_meta: BTreeMap::new(),
            default: default.clone(),
        }
    }

    /// Checks wether the database for the respective directory is still
    /// up to date.
    /// Will only return true if the database should be (re)read, i.e. it will
    /// return false if there is no database file in the specified directory.
    fn check_outdated(&self, db_dir: &PathBuf) -> bool {
        let mut db = db_dir.clone();
        db.push(".lang");
        let db = db.as_path();

        if let Ok(metadata) = db.metadata() {
            if !metadata.is_file() {
                // it exists, but it is a directory
                false
            } else if let (Ok(modified), Some(last_read)) =
                (metadata.modified(), self.databases_read.get(db))
            {
                // check that it was last modified before the read
                // if the times are the same, we might have read the old file
                &modified < last_read
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
        let mut db = db_dir.clone();
        db.push(".lang");
        let db = db.as_path();

        if let Ok(file) = std::fs::File::open(db) {
            let r = BufReader::new(file);
            r.lines()
                // discard any I/O errors
                .filter_map(|line| line.ok())
                // filter out comment lines
                .filter(|line| !line.trim_start().starts_with("#"))
                .for_each(|line| {
                    // split line at colon
                    let parts = line.splitn(2, ':').collect::<Vec<_>>();
                    // only continue if line fits the format
                    if parts.len() == 2 {
                        // generate workspace-unique path
                        let mut path = db_dir.clone();
                        path.push(parts[0].trim());
                        self.file_meta.insert(path, parts[1].trim().to_string());
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
    pub fn get(&mut self, file: PathBuf) -> &str {
        let dir = file.parent().expect("no parent directory").to_path_buf();
        if self.check_outdated(&dir) {
            self.read_database(&dir);
        }

        self.file_meta.get(&file).unwrap_or(&self.default)
    }
}
