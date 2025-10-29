use std::{cmp::Ordering, fs, io, path::Path};
use chrono::{Datelike, TimeZone};
use users::{get_group_by_gid, get_user_by_uid};
use std::os::unix::fs::{MetadataExt, PermissionsExt};

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
    for error in errors {
        eprintln!("{}", error);
    }

    let mut output = String::new();
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
            output.push_str(&format!("{}:\n", target));
        }

        // Handle single file or symlink
        if path.is_file() || (path.is_symlink() && !path.is_dir()) {
            if let Err(e) = list_file(path, target, config, output) {
                output.push_str(&format!("ls: {}: {}\n", target, e));
            }
            continue;
        }

        // Handle directory
        if let Err(e) = list_directory(path, config, output) {
            output.push_str(&format!("ls: {}: {}\n", target, e));
        }
    }
}

fn list_file(path: &Path, name: &str, config: &LsConfig, output: &mut String) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    let display_name = if config.classify {
        format!("{}{}", name, suffix_for(path, &meta))
    } else {
        name.to_string()
    };

    if config.long_format {
        output.push_str(&format!("{}\n", long_format_line(path, &meta, &display_name)));
    } else {
        output.push_str(&format!("{}\n", display_name));
    }
    Ok(())
}

fn list_directory(path: &Path, config: &LsConfig, output: &mut String) -> io::Result<()> {
    let mut items: Vec<(String, std::path::PathBuf, fs::Metadata)> = Vec::new();

    // Add . and ..
    if config.show_all {
        for name in &[".", ".."] {
            let p = path.join(name);
            if let Ok(meta) = fs::symlink_metadata(&p) {
                items.push((name.to_string(), p, meta));
            }
        }
    }

    // Read directory entries
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let name = entry.file_name().to_string_lossy().to_string();
        
        if !config.show_all && name.starts_with('.') {
            continue;
        }

        let item_path = entry.path();
        if let Ok(meta) = fs::symlink_metadata(&item_path) {
            items.push((name, item_path, meta));
        }
    }

    // Sort items
    items.sort_by(|a, b| ls_cmp(&a.0, &b.0));

    // Print total blocks for long format
    if config.long_format {
        let total_blocks: u64 = items.iter().map(|(_, _, m)| m.blocks() as u64).sum();
        output.push_str(&format!("total {}\n", (total_blocks + 1) / 2));
    }

    if config.long_format {
        for (name, path, meta) in &items {
            output.push_str(&format!("{}\n", long_format_line(path, meta, name)));
        }
    } else {
        let short_names: Vec<String> = items.iter().map(|(name, path, meta)| {
            if config.classify {
                format!("{}{}", name, suffix_for(path, meta))
            } else {
                name.clone()
            }
        }).collect();
        
        format_columns(&short_names, output);
    }

    Ok(())
}

fn format_columns(names: &[String], output: &mut String) {
    if names.is_empty() {
        return;
    }

    const TERM_WIDTH: usize = 80;
    const MIN_GAP: usize = 2;

    // Find max width
    let max_width = names.iter().map(|s| s.len()).max().unwrap_or(0);
    let col_width = max_width + MIN_GAP;

    // Calculate number of columns
    let num_cols = if col_width > 0 {
        (TERM_WIDTH / col_width).max(1)
    } else {
        1
    };

    // Calculate number of rows
    let num_rows = (names.len() + num_cols - 1) / num_cols;

    // Print in column-major order
    for row in 0..num_rows {
        for col in 0..num_cols {
            let idx = col * num_rows + row;
            if idx < names.len() {
                let name = &names[idx];
                if col < num_cols - 1 && idx + num_rows < names.len() {
                    output.push_str(&format!("{:<width$}", name, width = col_width));
                } else {
                    output.push_str(name);
                }
            }
        }
        output.push('\n');
    }
}

fn strip_dot(s: &str) -> &str {
    s.strip_prefix('.').unwrap_or(s)
}

fn ls_cmp(a: &str, b: &str) -> Ordering {
    let a_key = strip_dot(a).to_lowercase();
    let b_key = strip_dot(b).to_lowercase();
    a_key.as_bytes().cmp(b_key.as_bytes())
}

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
    let mut s = String::new();
    let bits = [
        0o400, 0o200, 0o100,
        0o040, 0o020, 0o010,
        0o004, 0o002, 0o001,
    ];
    let chars = ['r', 'w', 'x'];
    
    for i in 0..9 {
        s.push(if mode & bits[i] != 0 { chars[i % 3] } else { '-' });
    }

    // Handle special bits (setuid, setgid, sticky)
    let mut chars_vec: Vec<char> = s.chars().collect();
    if mode & 0o4000 != 0 {
        chars_vec[2] = if mode & 0o100 != 0 { 's' } else { 'S' };
    }
    if mode & 0o2000 != 0 {
        chars_vec[5] = if mode & 0o010 != 0 { 's' } else { 'S' };
    }
    if mode & 0o1000 != 0 {
        chars_vec[8] = if mode & 0o001 != 0 { 't' } else { 'T' };
    }
    
    chars_vec.into_iter().collect()
}

fn suffix_for<'a>(path: &'a Path, meta: &'a fs::Metadata) -> &'a str {
    let ft = meta.file_type();
    
    if ft.is_symlink() {
        if let Ok(target_meta) = fs::metadata(path) {
            if target_meta.is_dir() {
                return "/";
            } else if target_meta.is_file() && (target_meta.mode() & 0o111 != 0) {
                return "*";
            }
        }
        return "@";
    }
    
    if ft.is_dir() {
        "/"
    } else if ft.is_file() && (meta.mode() & 0o111 != 0) {
        "*"
    } else {
        match meta.mode() & 0o170000 {
            0o010000 => "|", // FIFO
            0o140000 => "=", // Socket
            _ => "",
        }
    }
}

fn long_format_line(path: &Path, meta: &fs::Metadata, name: &str) -> String {
    let file_type = file_type_char(meta);
    let perms = permissions_string(meta);
    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();
    
    let user = get_user_by_uid(uid)
        .map(|u| u.name().to_string_lossy().to_string())
        .unwrap_or_else(|| uid.to_string());
    
    let group = get_group_by_gid(gid)
        .map(|g| g.name().to_string_lossy().to_string())
        .unwrap_or_else(|| gid.to_string());

    let datetime = chrono::Local.timestamp_opt(meta.mtime(), 0).single().unwrap();
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
        },
        _ => format!("{:>8}", meta.len()),
    };

    let mut display_name = name.to_string();
    if meta.file_type().is_symlink() {
        if let Ok(target_path) = fs::read_link(path) {
            display_name.push_str(&format!(" -> {}", target_path.to_string_lossy()));
        }
    }
    display_name.push_str(suffix_for(path, meta));

    format!(
        "{}{} {:>2} {:<8} {:<8} {} {} {}",
        file_type, perms, nlink, user, group, size_or_dev, date_str, display_name
    )
}