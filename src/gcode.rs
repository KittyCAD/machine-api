pub struct GcodeSequence {
    pub lines: Vec<String>,
}

impl GcodeSequence {
    pub fn from_file_path(path: &str) -> Self {
        Self {
            lines: std::fs::read_to_string(path)
                .unwrap() // panic on possible file-reading errors
                .lines() // split the string into an iterator of string slices
                .map(|s| {
                    let s = String::from(s);
                    match s.split_once(';') {
                        Some((command, _)) => command.trim().to_string(),
                        None => s.trim().to_string(),
                    }
                }) // make each slice into a string
                .collect(),
        }
    }
}
