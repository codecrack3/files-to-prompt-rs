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
    pub path_template: Option<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dir_path: ".".to_string(),
            file_patterns: vec![
                "txt".to_string(), "py".to_string(), "c".to_string(), 
                "go".to_string(), "rs".to_string(), "java".to_string(),
                "js".to_string(), "html".to_string(), "ts".to_string()
            ],
            output_file: String::new(),
            lines: None,
            clean_input_enabled: Some(false),
            files: None,
            ignored_files: vec!["gitignore".to_string(), "git".to_string()],
            ignored_folders: vec![".git".to_string(), ".idea".to_string(), "node_modules".to_string()],
            template: "default".to_string(),
            path_template: None,
        }
    }
}
