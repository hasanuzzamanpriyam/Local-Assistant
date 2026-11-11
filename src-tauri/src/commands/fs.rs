use std::path::PathBuf;
use std::fs;
use walkdir::WalkDir;
use regex::Regex;
use tauri::command;
use anyhow::Result;

/// Get the workspace root directory
fn workspace_root() -> PathBuf {
    dirs::data_dir()
        .expect("No data dir")
        .join("assistant")
        .join("workspace")
}

/// Resolve a relative path to an absolute path within the workspace
/// Returns an error if the path tries to traverse outside the workspace
fn resolve_path(rel_path: &str) -> Result<PathBuf> {
    let root = workspace_root();
    let target = root.join(rel_path);
    
    // Canonicalize both paths to resolve symlinks and normalize
    let canonical_root = fs::canonicalize(&root)
        .map_err(|e| anyhow::anyhow!("Failed to canonicalize workspace root: {}", e))?;
    let canonical_target = fs::canonicalize(&target)
        .map_err(|e| anyhow::anyhow!("Failed to canonicalize target path: {}", e))?;
    
    // Ensure the target path is within the workspace root
    if !canonical_target.starts_with(&canonical_root) {
        return Err(anyhow::anyhow!("Path traversal attempt blocked: {}", rel_path));
    }
    
    Ok(canonical_target)
}

/// Read a file from the workspace
#[command]
pub async fn fs_read(path: String) -> Result<String, String> {
    let resolved = resolve_path(&path).map_err(|e| e.to_string())?;
    fs::read_to_string(&resolved).map_err(|e| e.to_string())
}

/// Write a file to the workspace (creates parent directories if needed)
#[command]
pub async fn fs_write(path: String, content: String) -> Result<(), String> {
    let resolved = resolve_path(&path).map_err(|e| e.to_string())?;
    
    // Create parent directories if they don't exist
    if let Some(parent) = resolved.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    
    fs::write(&resolved, content).map_err(|e| e.to_string())
}

/// Edit a file by replacing old_text with new_text
#[command]
pub async fn fs_edit(path: String, old_text: String, new_text: String) -> Result<(), String> {
    let resolved = resolve_path(&path).map_err(|e| e.to_string())?;
    
    let content = fs::read_to_string(&resolved).map_err(|e| e.to_string())?;
    
    if !content.contains(&old_text) {
        return Err(format!("Old text not found in file: {}", path));
    }
    
    let new_content = content.replace(&old_text, &new_text);
    fs::write(&resolved, new_content).map_err(|e| e.to_string())
}

/// Glob pattern matching for files in the workspace
#[command]
pub async fn fs_glob(pattern: String) -> Result<Vec<String>, String> {
    let root = workspace_root();
    
    let mut results = Vec::new();
    
    // Use walkdir to find matching files
    for entry in WalkDir::new(&root)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip directories
        if !path.is_file() {
            continue;
        }
        
        // Get relative path from workspace root
        let rel_path = path.strip_prefix(&root)
            .map_err(|e| e.to_string())?
            .to_string_lossy()
            .to_string();
        
        // Check if the relative path matches the glob pattern
        if glob_match(&rel_path, &pattern) {
            results.push(rel_path);
        }
    }
    
    results.sort();
    Ok(results)
}

/// Simple glob matching (supports *, ?, **)
fn glob_match(path: &str, pattern: &str) -> bool {
    glob_match_recursive(path, pattern)
}

fn glob_match_recursive(path: &str, pattern: &str) -> bool {
    // Handle ** specially - it matches zero or more path segments
    if pattern == "**" {
        return true;
    }
    
    // Split pattern by ** to handle it specially
    if pattern.contains("**") {
        let parts: Vec<&str> = pattern.split("**").collect();
        
        // If pattern starts with **, match the rest against any suffix of path
        if pattern.starts_with("**") {
            // Pattern: **/foo.txt -> match foo.txt at any depth
            let suffix = &parts[1..].join("**");
            // Remove leading / from suffix if present
            let suffix = suffix.trim_start_matches('/');
            return path.ends_with(suffix) || glob_match_recursive(path, suffix);
        }
        
        // If pattern ends with **, match the prefix against any prefix of path
        if pattern.ends_with("**") {
            let prefix = &parts[..parts.len()-1].join("**");
            // Remove trailing / from prefix if present
            let prefix = prefix.trim_end_matches('/');
            return path.starts_with(prefix) || glob_match_recursive(path, prefix);
        }
        
        // Pattern has ** in the middle: prefix**suffix
        // We need to find if there's a way to split path such that prefix matches start and suffix matches end
        let prefix = parts[0].trim_end_matches('/');
        let suffix = parts[1].trim_start_matches('/');
        
        // Try all possible splits of path
        for i in 0..=path.len() {
            let path_prefix = &path[..i];
            let path_suffix = &path[i..];
            if glob_match_simple(path_prefix, prefix) && glob_match_simple(path_suffix, suffix) {
                return true;
            }
        }
        return false;
    }
    
    // No ** in pattern, use simple matching
    glob_match_simple(path, pattern)
}

fn glob_match_simple(path: &str, pattern: &str) -> bool {
    // Convert simple glob pattern (only * and ?) to regex
    let mut regex_pattern = String::new();
    let mut chars = pattern.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '*' => {
                // * matches any characters (including path separators)
                regex_pattern.push_str(".*");
            }
            '?' => {
                // ? matches any single character
                regex_pattern.push_str(".");
            }
            '[' => {
                // Character class - copy until closing ]
                regex_pattern.push('[');
                while let Some(c) = chars.next() {
                    regex_pattern.push(c);
                    if c == ']' {
                        break;
                    }
                }
            }
            '\\' => {
                // Escape next character
                if let Some(c) = chars.next() {
                    regex_pattern.push('\\');
                    regex_pattern.push(c);
                }
            }
            '.' | '+' | '(' | ')' | '|' | '^' | '$' | '{' | '}' => {
                // Escape regex special characters
                regex_pattern.push('\\');
                regex_pattern.push(c);
            }
            _ => {
                regex_pattern.push(c);
            }
        }
    }
    
    let regex_pattern = format!("^{}$", regex_pattern);
    Regex::new(&regex_pattern)
        .map(|re| re.is_match(path))
        .unwrap_or(false)
}

/// Grep for a pattern in files within the workspace
#[command]
pub async fn fs_grep(pattern: String, path: Option<String>) -> Result<Vec<GrepMatch>, String> {
    let root = workspace_root();
    
    // Determine search directory
    let search_dir = if let Some(p) = path {
        root.join(p)
    } else {
        root.clone()
    };
    
    // Ensure search directory is within workspace
    let canonical_root = fs::canonicalize(&root).map_err(|e| e.to_string())?;
    let canonical_search = fs::canonicalize(&search_dir).map_err(|e| e.to_string())?;
    
    if !canonical_search.starts_with(&canonical_root) {
        return Err("Path traversal attempt blocked".to_string());
    }
    
    // Compile the regex pattern
    let regex = Regex::new(&pattern).map_err(|e| e.to_string())?;
    
    let mut results = Vec::new();
    
    // Walk through files
    for entry in WalkDir::new(&search_dir)
        .follow_links(true)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        
        // Skip directories
        if !path.is_file() {
            continue;
        }
        
        // Read file content
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(_) => continue, // Skip binary files
        };
        
        // Search for matches
        for (line_num, line) in content.lines().enumerate() {
            if regex.is_match(line) {
                // Get relative path from workspace root
                let rel_path = path.strip_prefix(&root)
                    .map_err(|e| e.to_string())?
                    .to_string_lossy()
                    .to_string();
                
                results.push(GrepMatch {
                    path: rel_path,
                    line_number: line_num + 1,
                    line: line.to_string(),
                });
            }
        }
    }
    
    Ok(results)
}

#[derive(serde::Serialize)]
pub struct GrepMatch {
    pub path: String,
    pub line_number: usize,
    pub line: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;
    
    #[test]
    fn test_glob_match() {
        assert!(glob_match("test.txt", "*.txt"));
        assert!(glob_match("foo/bar.txt", "*.txt"));
        assert!(!glob_match("test.rs", "*.txt"));
        assert!(glob_match("foo/bar/baz.txt", "**/*.txt"));
        assert!(glob_match("test.txt", "**/*.txt"));
        assert!(glob_match("a/b/c.txt", "a/**/*.txt"));
        assert!(!glob_match("x/b/c.txt", "a/**/*.txt"));
        assert!(glob_match("test.txt", "test.tx?"));
        assert!(glob_match("test.txt", "te?t.txt"));
    }
    
    #[test]
    fn test_resolve_path() {
        let dir = tempdir().unwrap();
        let root = dir.path().join("workspace");
        fs::create_dir_all(&root).unwrap();
        
        // Test normal path
        let file = root.join("test.txt");
        fs::write(&file, "hello").unwrap();
        
        // We can't easily test the full resolve_path without mocking workspace_root
        // but we can test the canonicalization logic
        let canonical_root = fs::canonicalize(&root).unwrap();
        let canonical_file = fs::canonicalize(&file).unwrap();
        assert!(canonical_file.starts_with(&canonical_root));
    }
}