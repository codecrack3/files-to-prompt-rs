use crate::model::Config;
use rayon::prelude::*;
use std::fs;
use std::path::PathBuf;

pub fn traverse(dir: PathBuf, files_extension: Vec<String>, config: Config) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut dirs = vec![dir];

    while !dirs.is_empty() {
        let (new_dirs, new_files): (Vec<_>, Vec<_>) = dirs
            .par_iter()
            .map(|dir| {
                let mut f = Vec::new();
                let mut d = Vec::new();
                let entries = fs::read_dir(dir).unwrap();
                for entry in entries {
                    let entry = entry.unwrap();
                    let path = entry.path();

                    if path.is_dir()
                        && !config
                            .ignored_folders
                            .contains(&path.file_name().unwrap().to_str().unwrap().to_string())
                    {
                        d.push(path);
                    } else {
                        if let Some(extension) = path.extension() {
                            if files_extension.contains(&extension.to_str().unwrap().to_string())
                                && !config
                                    .ignored_files
                                    .contains(&extension.to_str().unwrap().to_string())
                            {
                                f.push(path);
                            }
                        }
                    }
                }
                (d, f)
            })
            .unzip();
        files.extend(new_files.into_iter().flatten());
        dirs = new_dirs.into_iter().flatten().collect();
    }
    files
}
