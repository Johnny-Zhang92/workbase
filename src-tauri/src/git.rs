use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
pub struct GitStatus {
    pub branch: String,
    pub files: Vec<GitFileEntry>,
}

#[derive(Debug, Clone, Serialize)]
pub struct GitFileEntry {
    pub path: String,
    pub status: char,
}

pub fn get_git_status(root_path: &str) -> Result<GitStatus, String> {
    let branch = get_branch(root_path).unwrap_or_default();
    let files = get_status_files(root_path).unwrap_or_default();
    Ok(GitStatus { branch, files })
}

fn get_branch(repo: &str) -> Result<String, String> {
    let output = std::process::Command::new("git")
        .args(["-C", repo, "rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map_err(|e| format!("git rev-parse: {}", e))?;
    if output.status.success() {
        let name = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if name.is_empty() {
            Err("detached HEAD".into())
        } else {
            Ok(name)
        }
    } else {
        Err(String::from_utf8_lossy(&output.stderr).trim().to_string())
    }
}

fn parse_porcelain_line(line: &str, repo: &str) -> Option<GitFileEntry> {
    if line.len() < 4 {
        return None;
    }
    let bytes = line.as_bytes();
    let x = bytes[0] as char; // staged
    let y = bytes[1] as char; // unstaged

    let status = if y != ' ' { y } else { x };
    if status == ' ' || status == '!' {
        return None; // unmodified or ignored — skip
    }

    let rest = line[3..].trim();
    let path = if x == 'R' || y == 'R' {
        rest.split(" -> ").last().unwrap_or(rest).to_string()
    } else {
        rest.to_string()
    };

    let abs_path = std::path::Path::new(repo).join(&path);
    let abs_path = abs_path.to_string_lossy().replace('\\', "/");

    Some(GitFileEntry { path: abs_path, status })
}

fn get_status_files(repo: &str) -> Result<Vec<GitFileEntry>, String> {
    let output = std::process::Command::new("git")
        .args(["-C", repo, "status", "--porcelain=v1"])
        .output()
        .map_err(|e| format!("git status: {}", e))?;
    if !output.status.success() {
        return Ok(Vec::new());
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    Ok(stdout.lines().filter_map(|l| parse_porcelain_line(l, repo)).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    const REPO: &str = "/home/user/repo";

    #[test]
    fn test_parse_modified() {
        let entry = parse_porcelain_line(" M src/main.rs", REPO).unwrap();
        assert_eq!(entry.status, 'M');
        assert!(entry.path.ends_with("src/main.rs"));
    }

    #[test]
    fn test_parse_added() {
        let entry = parse_porcelain_line("A  new_file.txt", REPO).unwrap();
        assert_eq!(entry.status, 'A');
        assert!(entry.path.ends_with("new_file.txt"));
    }

    #[test]
    fn test_parse_deleted() {
        let entry = parse_porcelain_line(" D old_file.txt", REPO).unwrap();
        assert_eq!(entry.status, 'D');
        assert!(entry.path.ends_with("old_file.txt"));
    }

    #[test]
    fn test_parse_renamed() {
        let entry = parse_porcelain_line("R  old.txt -> new.txt", REPO).unwrap();
        assert_eq!(entry.status, 'R');
        assert!(entry.path.ends_with("new.txt"));
    }

    #[test]
    fn test_parse_staged_modified() {
        let entry = parse_porcelain_line("M  staged.txt", REPO).unwrap();
        assert_eq!(entry.status, 'M');
        assert!(entry.path.ends_with("staged.txt"));
    }

    #[test]
    fn test_parse_both_modified() {
        // MM: staged and unstaged modifications — y takes precedence
        let entry = parse_porcelain_line("MM double.txt", REPO).unwrap();
        assert_eq!(entry.status, 'M');
        assert!(entry.path.ends_with("double.txt"));
    }

    #[test]
    fn test_skip_unmodified() {
        assert!(parse_porcelain_line("   unchanged.txt", REPO).is_none());
    }

    #[test]
    fn test_skip_ignored() {
        assert!(parse_porcelain_line("!! ignored.txt", REPO).is_none());
    }

    #[test]
    fn test_skip_short_line() {
        assert!(parse_porcelain_line(" M", REPO).is_none());
        assert!(parse_porcelain_line("", REPO).is_none());
    }

    #[test]
    fn test_parse_untracked() {
        let entry = parse_porcelain_line("?? new_file.rs", REPO).unwrap();
        assert_eq!(entry.status, '?');
        assert!(entry.path.ends_with("new_file.rs"));
    }

    #[test]
    fn test_parse_unstaged_modified() {
        let entry = parse_porcelain_line(" M changed.rs", REPO).unwrap();
        assert_eq!(entry.status, 'M');
        assert!(entry.path.ends_with("changed.rs"));
    }

    #[test]
    fn test_path_uses_forward_slash() {
        let entry = parse_porcelain_line(" M sub/dir/file.rs", REPO).unwrap();
        assert!(entry.path.contains('/'));
        assert!(!entry.path.contains('\\'));
    }
}
