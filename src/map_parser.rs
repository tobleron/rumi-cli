use std::fs;

pub struct MapParser;

impl MapParser {
    pub fn get_context_map() -> String {
        match fs::read_to_string("MAP.md") {
            Ok(content) => content,
            Err(_) => "MAP.md not found. Proceeding without map.".to_string(),
        }
    }

    /// Returns a list of all known file paths from the map
    #[allow(dead_code)]
    pub fn get_known_files() -> Vec<String> {
        let content = Self::get_context_map();
        let mut files = Vec::new();
        
        for line in content.lines() {
            if let Some(start) = line.find('[') {
                if let Some(end) = line.find(']') {
                    // Extract [src/Main.res] -> src/Main.res
                    let path = &line[start + 1..end];
                    if path.contains('.') { // Basic filter for file-like strings
                        files.push(path.to_string());
                    }
                }
            }
        }
        files
    }
}
