use chrono::{Datelike, TimeZone};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{cmp::Ordering, fs, io, path::Path};
use users::{get_group_by_gid, get_user_by_uid};
use terminal_size::{Width, terminal_size};

/// ANSI color codes
const BLUE: &str = "\x1b[34m";
const GREEN: &str = "\x1b[32m";
const CYAN: &str = "\x1b[36m";
const MAGENTA: &str = "\x1b[35m";
const YELLOW: &str = "\x1b[33m";
const RESET: &str = "\x1b[0m";

/// Configuration for ls behavior
#[derive(Default)]
struct LsConfig {
    show_all: bool,
    long_format: bool,
    classify: bool,
}

pub fn lss(flags: &[String], args: &[String]) {
    let mut config = LsConfig::default();
    let mut errors = Vec::new();

    // Parse flags
    for flag in flags {
        match flag.as_str() {
            "a" => config.show_all = true,
            "l" => config.long_format = true,
            "F" => config.classify = true,
            _ => errors.push(format!("ls: invalid option -- '{}'", flag)),
        }
    }

    let targets: Vec<&str> = if args.is_empty() {
        vec!["."]
    } else {
        args.iter().map(|s| s.as_str()).collect()
    };

    // Print errors first
    for error in &errors {
        eprintln!("{}", error);
    }

    let mut output = String::with_capacity(4096);
    list_targets(&targets, &config, &mut output);

    if output.ends_with('\n') {
        output.pop();
    }
    println!("{}", output);
}

fn list_targets(targets: &[&str], config: &LsConfig, output: &mut String) {
    let show_header = targets.len() > 1;

    for (idx, target) in targets.iter().enumerate() {
        if idx > 0 {
            output.push('\n');
        }

        let path = Path::new(target);

        if show_header {
            output.push_str(target);
            output.push_str(":\n");
        }

        // Handle single file or symlink
        if path.is_file() || (path.is_symlink() && !path.is_dir()) {
            if let Err(e) = list_file(path, target, config, output) {
                output.push_str("ls: ");
                output.push_str(target);
                output.push_str(": ");
                output.push_str(&e.to_string());
                output.push('\n');
            }
            continue;
        }

        // Handle directory
        if let Err(e) = list_directory(path, config, output) {
            output.push_str("ls: ");
            output.push_str(target);
            output.push_str(": ");
            output.push_str(&e.to_string());
            output.push('\n');
        }
    }
}

fn list_file(path: &Path, name: &str, config: &LsConfig, output: &mut String) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    let is_symlink = meta.file_type().is_symlink();

    let display_name = if config.long_format {
        if is_symlink {
            name.to_string()
        } else {
            let mut s = String::with_capacity(name.len() + 1);
            s.push_str(name);
            s.push_str(suffix_for( &meta));
            s
        }
    } else if config.classify {
        let mut s = String::with_capacity(name.len() + 1);
        s.push_str(name);
        s.push_str(suffix_for(&meta));
        s
    } else {
        name.to_string()
    };

    let colored_name = colorize(&display_name, &meta);

    if config.long_format {
        output.push_str(&long_format_line(path, &meta, &colored_name));
        output.push('\n');
    } else {
        output.push_str(&colored_name);
        output.push('\n');
    }
    Ok(())
}

fn list_directory(path: &Path, config: &LsConfig, output: &mut String) -> io::Result<()> {
    let mut items: Vec<(String, std::path::PathBuf, fs::Metadata)> = Vec::new();

    if config.show_all {
        for name in &[".", ".."] {
            let p = path.join(name);
            if let Ok(meta) = fs::symlink_metadata(&p) {
                items.push((name.to_string(), p, meta));
            }
        }
    }

    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().into_owned();

        if !config.show_all && name.starts_with('.') {
            continue;
        }

        let item_path = entry.path();
        if let Ok(meta) = fs::symlink_metadata(&item_path) {
            items.push((name, item_path, meta));
        }
    }

    items.sort_unstable_by(|a, b| ls_cmp(&a.0, &b.0));

    if config.long_format {
        let total_blocks: u64 = items.iter().map(|(_, _, m)| m.blocks() as u64).sum();
        output.push_str("total ");
        output.push_str(&((total_blocks + 1) / 2).to_string());
        output.push('\n');

        for (name, path, meta) in &items {
            let colored = colorize(name, meta);
            output.push_str(&long_format_line(path, meta, &colored));
            output.push('\n');
        }
    } else {
        let short_names: Vec<String> = items
            .iter()
            .map(|(name, _path, meta)| {
                let mut n = String::with_capacity(name.len() + 1);
                n.push_str(name);
                if config.classify {
                    n.push_str(suffix_for( meta));
                }
                colorize(&n, meta)
            })
            .collect();

        format_columns(&short_names, output);
    }

    Ok(())
}

/// Apply ANSI colors similar to `ls --color`
fn colorize(name: &str, meta: &fs::Metadata) -> String {
    let ft = meta.file_type();
    let mode = meta.mode();

    if ft.is_symlink() {
        format!("{CYAN}{name}{RESET}")
    } else if ft.is_dir() {
        format!("{BLUE}{name}{RESET}")
    } else if mode & 0o111 != 0 {
        format!("{GREEN}{name}{RESET}")
    } else if (mode & 0o170000) == 0o010000 {
        format!("{YELLOW}{name}{RESET}")
    } else if (mode & 0o170000) == 0o140000 {
        format!("{MAGENTA}{name}{RESET}")
    } else {
        name.to_string()
    }
}

// Add this to your Cargo.toml dependencies:
// terminal_size = "0.3"

fn format_columns(names: &[String], output: &mut String) {
    if names.is_empty() {
        return;
    }

    const MIN_GAP: usize = 2;
    const FALLBACK_WIDTH: usize = 80;

    // Get actual terminal width
    let term_width = terminal_size()
        .map(|(Width(w), _)| w as usize)
        .unwrap_or(FALLBACK_WIDTH);

    // Strip ANSI codes for width calculation
    fn visible_width(s: &str) -> usize {
        let mut width = 0;
        let mut in_escape = false;
        for c in s.chars() {
            if c == '\x1b' {
                in_escape = true;
            } else if in_escape && c == 'm' {
                in_escape = false;
            } else if !in_escape {
                width += 1;
            }
        }
        width
    }

    // Calculate visible widths for all names
    let widths: Vec<usize> = names.iter().map(|s| visible_width(s)).collect();
    let max_width = *widths.iter().max().unwrap_or(&0);
    
    // If even one item won't fit, print one per line
    if max_width >= term_width {
        for name in names {
            output.push_str(name);
            output.push('\n');
        }
        return;
    }

    // Calculate optimal number of columns
    let col_width = max_width + MIN_GAP;
    let num_cols = (term_width / col_width).max(1);
    let num_rows = (names.len() + num_cols - 1) / num_cols;

    // Print in column-major order
    for row in 0..num_rows {
        for col in 0..num_cols {
            let idx = col * num_rows + row;
            if idx < names.len() {
                let name = &names[idx];
                output.push_str(name);
                
                // Add padding for all but last column
                if col < num_cols - 1 {
                    let visible = widths[idx];
                    let padding = col_width.saturating_sub(visible);
                    for _ in 0..padding {
                        output.push(' ');
                    }
                }
            }
        }
        output.push('\n');
    }
}


#[inline]
fn ls_cmp(a: &str, b: &str) -> Ordering {
    let a_dot = a.starts_with('.');
    let b_dot = b.starts_with('.');

    match (a_dot, b_dot) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

#[inline]
fn suffix_for<'a>(meta: &'a fs::Metadata) -> &'a str {
    let ft = meta.file_type();

    if ft.is_symlink() {
        return "@";
    }

    if ft.is_dir() {
        "/"
    } else if ft.is_file() && (meta.mode() & 0o111 != 0) {
        "*"
    } else {
        match meta.mode() & 0o170000 {
            0o010000 => "|",
            0o140000 => "=",
            _ => "",
        }
    }
}

#[inline]
fn file_type_char(meta: &fs::Metadata) -> char {
    match meta.mode() & 0o170000 {
        0o040000 => 'd',
        0o100000 => '-',
        0o120000 => 'l',
        0o010000 => 'p',
        0o060000 => 'b',
        0o020000 => 'c',
        0o140000 => 's',
        _ => '?',
    }
}

fn permissions_string(meta: &fs::Metadata) -> String {
    let mode = meta.permissions().mode();
    let mut chars = ['-'; 9];
    
    const BITS: [u32; 9] = [0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001];
    const CHAR_MAP: [char; 3] = ['r', 'w', 'x'];

    for (i, &bit) in BITS.iter().enumerate() {
        if mode & bit != 0 {
            chars[i] = CHAR_MAP[i % 3];
        }
    }

    // Handle special bits
    if mode & 0o4000 != 0 {
        chars[2] = if mode & 0o100 != 0 { 's' } else { 'S' };
    }
    if mode & 0o2000 != 0 {
        chars[5] = if mode & 0o010 != 0 { 's' } else { 'S' };
    }
    if mode & 0o1000 != 0 {
        chars[8] = if mode & 0o001 != 0 { 't' } else { 'T' };
    }

    chars.iter().collect()
}

fn long_format_line(path: &Path, meta: &fs::Metadata, name: &str) -> String {
    let file_type = file_type_char(meta);
    let perms = permissions_string(meta);
    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();

    let user = get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| uid.to_string());

    let group = get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().into_owned())
        .unwrap_or_else(|| gid.to_string());

    let datetime = chrono::Local
        .timestamp_opt(meta.mtime(), 0)
        .single()
        .unwrap();
    let now = chrono::Local::now();
    let date_str = if datetime.year() == now.year() {
        datetime.format("%b %e %H:%M").to_string()
    } else {
        datetime.format("%b %e  %Y").to_string()
    };

    let size_or_dev = match file_type {
        'c' | 'b' => {
            let rdev = meta.rdev();
            format!("{:>3}, {:>3}", (rdev >> 8) & 0xff, rdev & 0xff)
        }
        _ => format!("{:>8}", meta.len()),
    };

    let mut display_name = String::with_capacity(name.len() + 20);
    display_name.push_str(name);
    
    if meta.file_type().is_symlink() {
        if let Ok(target_path) = fs::read_link(path) {
            display_name.push_str(" -> ");
            display_name.push_str(&target_path.to_string_lossy());
        }
    } else {
        display_name.push_str(suffix_for( meta));
    }

    format!(
        "{}{} {:>2} {:<8} {:<8} {} {} {}",
        file_type, perms, nlink, user, group, size_or_dev, date_str, display_name
    )
}