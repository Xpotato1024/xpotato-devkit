use std::fs;
use std::path::Path;

lazy_static::lazy_static! {
    static ref FRONTMATTER_RE: regex::Regex = regex::Regex::new(r"(?m)\A---\r?\n").unwrap();
}

pub fn split_frontmatter(text: &str) -> (&str, &str) {
    if !text.starts_with("---") {
        return ("", text);
    }

    if let Some(pos) = text[3..].find("\n---") {
        let abs_pos = pos + 3;
        if let Some(close_pos) = text[abs_pos + 1..].find("---") {
            let actual_close = abs_pos + 1 + close_pos;
            let mut after_close = actual_close + 3;
            // consume trailing newline(s) of closing ---
            let bytes = text.as_bytes();
            while after_close < bytes.len()
                && (bytes[after_close] == b'\r' || bytes[after_close] == b'\n')
            {
                after_close += 1;
            }
            return (&text[..after_close], &text[after_close..]);
        }
    }

    ("", text)
}

fn heading_level(line: &str) -> usize {
    let stripped = line.trim();
    if !stripped.starts_with("#") {
        return 0;
    }
    let parts: Vec<&str> = stripped.split_whitespace().collect();
    if parts.is_empty() {
        return 0;
    }
    let marker = parts[0];
    for ch in marker.chars() {
        if ch != '#' {
            return 0;
        }
    }
    marker.len()
}

fn heading_text(line: &str) -> String {
    let stripped = line.trim();
    let level = heading_level(stripped);
    if level == 0 {
        return String::new();
    }
    stripped[level..].trim().to_string()
}

fn find_section(
    lines: &[&str],
    heading: &str,
    exact: bool,
) -> Result<(usize, usize, usize), String> {
    let mut matches = Vec::new();
    for (i, &line) in lines.iter().enumerate() {
        let level = heading_level(line);
        if level == 0 {
            continue;
        }

        let text = heading_text(line);
        let full = line.trim();

        if exact {
            if text == heading || full == heading {
                matches.push((i, level));
            }
        } else {
            if full.contains(heading) {
                matches.push((i, level));
            }
        }
    }

    if matches.is_empty() {
        return Err(format!("Heading '{}' not found.", heading));
    }
    if matches.len() > 1 {
        let locs: Vec<_> = matches
            .iter()
            .map(|(idx, _)| format!("L{}", idx + 1))
            .collect();
        return Err(format!(
            "Heading '{}' matched {} sections ({}). Provide a more specific heading.",
            heading,
            matches.len(),
            locs.join(", ")
        ));
    }

    let (start_idx, level) = matches[0];
    let content_start = start_idx + 1;
    let mut end_idx = lines.len();

    for i in (start_idx + 1)..lines.len() {
        let cur_level = heading_level(lines[i]);
        if cur_level > 0 && cur_level <= level {
            end_idx = i;
            break;
        }
    }

    Ok((start_idx, content_start, end_idx))
}

fn get_lines<'a>(body: &'a str) -> Vec<&'a str> {
    let mut raw_lines = Vec::new();
    let mut last = 0;
    for (i, _) in body.match_indices('\n') {
        raw_lines.push(&body[last..=i]);
        last = i + 1;
    }
    if last < body.len() {
        raw_lines.push(&body[last..]);
    }
    raw_lines
}

pub fn append_to_section(
    filepath: &Path,
    heading: &str,
    mut content: String,
    dry_run: bool,
) -> Result<String, String> {
    let text = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    let (frontmatter, body) = split_frontmatter(&text);
    let lines = get_lines(body);
    let plain_lines: Vec<_> = lines
        .iter()
        .map(|l| l.trim_end_matches('\n').trim_end_matches('\r'))
        .collect();

    let (_, _, end_idx) = find_section(&plain_lines, heading, true)?;

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    let mut result = String::from(frontmatter);
    for line in &lines[..end_idx] {
        result.push_str(line);
    }
    result.push_str(&content);
    for line in &lines[end_idx..] {
        result.push_str(line);
    }

    if !dry_run {
        fs::write(filepath, &result).map_err(|e| e.to_string())?;
    }
    Ok(result)
}

pub fn replace_section(
    filepath: &Path,
    heading: &str,
    mut content: String,
    keep_heading: bool,
    dry_run: bool,
) -> Result<String, String> {
    let text = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    let (frontmatter, body) = split_frontmatter(&text);
    let lines = get_lines(body);
    let plain_lines: Vec<_> = lines
        .iter()
        .map(|l| l.trim_end_matches('\n').trim_end_matches('\r'))
        .collect();

    let (start_idx, content_start, end_idx) = find_section(&plain_lines, heading, true)?;

    if !content.is_empty() && !content.ends_with('\n') {
        content.push('\n');
    }

    let mut result = String::from(frontmatter);
    let to_idx = if keep_heading {
        content_start
    } else {
        start_idx
    };
    for line in &lines[..to_idx] {
        result.push_str(line);
    }
    result.push_str(&content);
    for line in &lines[end_idx..] {
        result.push_str(line);
    }

    if !dry_run {
        fs::write(filepath, &result).map_err(|e| e.to_string())?;
    }
    Ok(result)
}

pub fn ensure_section(
    filepath: &Path,
    heading: &str,
    mut content: String,
    level: usize,
    after: Option<&str>,
    dry_run: bool,
) -> Result<String, String> {
    let text = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    let (frontmatter, body) = split_frontmatter(&text);
    let lines = get_lines(body);
    let plain_lines: Vec<_> = lines
        .iter()
        .map(|l| l.trim_end_matches('\n').trim_end_matches('\r'))
        .collect();

    if find_section(&plain_lines, heading, true).is_ok() {
        return Ok(text);
    }

    let heading_line = format!("{} {}\n", "#".repeat(level), heading);
    let mut block = heading_line;
    if !content.is_empty() {
        if !content.ends_with('\n') {
            content.push('\n');
        }
        block.push_str(&content);
    }

    let mut result = String::from(frontmatter);
    if let Some(after_heading) = after {
        let (_, _, end_idx) = find_section(&plain_lines, after_heading, true)?;
        for line in &lines[..end_idx] {
            result.push_str(line);
        }
        result.push_str("\n");
        result.push_str(&block);
        for line in &lines[end_idx..] {
            result.push_str(line);
        }
    } else {
        for line in &lines {
            result.push_str(line);
        }
        if !lines.is_empty() && !lines.last().unwrap().trim().is_empty() {
            result.push_str("\n");
        }
        result.push_str(&block);
    }

    if !dry_run {
        fs::write(filepath, &result).map_err(|e| e.to_string())?;
    }
    Ok(result)
}

pub fn append_bullet(
    filepath: &Path,
    heading: &str,
    bullet: &str,
    dedupe: bool,
    dry_run: bool,
) -> Result<String, String> {
    let text = fs::read_to_string(filepath).map_err(|e| e.to_string())?;
    let (frontmatter, body) = split_frontmatter(&text);
    let lines = get_lines(body);
    let plain_lines: Vec<_> = lines
        .iter()
        .map(|l| l.trim_end_matches('\n').trim_end_matches('\r'))
        .collect();

    let (_, content_start, end_idx) = find_section(&plain_lines, heading, true)?;

    let mut bullet_stripped = bullet.trim().to_string();
    if !bullet_stripped.starts_with("- ") {
        bullet_stripped = format!("- {}", bullet_stripped);
    }

    if dedupe {
        for i in content_start..end_idx {
            if plain_lines[i].trim() == bullet_stripped {
                return Ok(text);
            }
        }
    }

    let mut bullet_line = bullet_stripped;
    bullet_line.push('\n');

    let mut result = String::from(frontmatter);
    for line in &lines[..end_idx] {
        result.push_str(line);
    }
    result.push_str(&bullet_line);
    for line in &lines[end_idx..] {
        result.push_str(line);
    }

    if !dry_run {
        fs::write(filepath, &result).map_err(|e| e.to_string())?;
    }
    Ok(result)
}
