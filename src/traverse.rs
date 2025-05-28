use crate::model::Config;
use rayon::prelude::*;
use std::collections::VecDeque;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

pub fn traverse(dir: PathBuf, file_extensions: Vec<String>, config: Config) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = VecDeque::new();
    dirs.push_back(dir);

    while let Some(current_dir) = dirs.pop_front() {
        if let Ok(entries) = fs::read_dir(&current_dir) {
            let (new_dirs, new_files): (Vec<_>, Vec<_>) = entries
                .filter_map(|entry| entry.ok())
                .collect::<Vec<_>>()
                .par_iter()
                .filter_map(|entry| {
                    let path = entry.path();
                    
                    if path.is_dir() {
                        if let Some(dir_name) = path.file_name().and_then(OsStr::to_str) {
                            if !config.ignored_folders.contains(&dir_name.to_string()) {
                                return Some((Some(path), None));
                            }
                        }
                    } else if path.is_file() {
                        if let Some(extension) = path.extension().and_then(OsStr::to_str) {
                            if file_extensions.contains(&extension.to_string()) 
                                && !config.ignored_files.contains(&extension.to_string()) {
                                return Some((None, Some(path)));
                            }
                        }
                    }
                    None
                })
                .unzip();

            // Add new directories to the queue
            dirs.extend(new_dirs.into_iter().flatten());
            
            // Add new files to the result
            files.extend(new_files.into_iter().flatten());
        }
    }

    files
}
