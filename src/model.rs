#[derive(Clone, Debug)]
pub struct Config {
    pub dir_path: String,
    pub file_patterns: Vec<String>,
    pub output_file: String,
    pub lines: Option<i32>,
    pub clean_input_enabled: Option<bool>,
    pub files: Option<Vec<String>>,
    pub ignored_files: Vec<String>,
    pub ignored_folders: Vec<String>,
    pub template: String,
}
