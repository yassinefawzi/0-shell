use chrono::{Datelike, TimeZone};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::{cmp::Ordering, fs, io, path::Path};
use users::{get_group_by_gid, get_user_by_uid};

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

    let display_name = if config.long_format {
        if meta.file_type().is_symlink() {
            name.to_string()
        } else {
            format!("{}{}", name, suffix_for(path, &meta))
        }
    } else if config.classify {
        format!("{}{}", name, suffix_for(path, &meta))
    } else {
        name.to_string()
    };

    // Apply color
    let colored_name = colorize(&display_name, &meta);

    if config.long_format {
        output.push_str(&format!(
            "{}\n",
            long_format_line(path, &meta, &colored_name)
        ));
    } else {
        output.push_str(&format!("{}\n", colored_name));
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
        let name = entry.file_name().to_string_lossy().to_string();

        if !config.show_all && name.starts_with('.') {
            continue;
        }

        let item_path = entry.path();
        if let Ok(meta) = fs::symlink_metadata(&item_path) {
            items.push((name, item_path, meta));
        }
    }

    items.sort_by(|a, b| ls_cmp(&a.0, &b.0));

    if config.long_format {
        let total_blocks: u64 = items.iter().map(|(_, _, m)| m.blocks() as u64).sum();
        output.push_str(&format!("total {}\n", (total_blocks + 1) / 2));
    }

    if config.long_format {
        for (name, path, meta) in &items {
            let colored = colorize(name, meta);
            output.push_str(&format!("{}\n", long_format_line(path, meta, &colored)));
        }
    } else {
        let short_names: Vec<String> = items
            .iter()
            .map(|(name, path, meta)| {
                let mut n = name.clone();
                if config.classify {
                    n.push_str(suffix_for(path, meta));
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

    if ft.is_symlink() {
        format!("{CYAN}{name}{RESET}")
    } else if ft.is_dir() {
        format!("{BLUE}{name}{RESET}")
    } else if meta.mode() & 0o111 != 0 {
        format!("{GREEN}{name}{RESET}")
    } else if (meta.mode() & 0o170000) == 0o010000 {
        format!("{YELLOW}{name}{RESET}")
    } else if (meta.mode() & 0o170000) == 0o140000 {
        format!("{MAGENTA}{name}{RESET}")
    } else {
        name.to_string()
    }
}

fn format_columns(names: &[String], output: &mut String) {
    if names.is_empty() {
        return;
    }

    const TERM_WIDTH: usize = 80;
    const MIN_GAP: usize = 2;

    let max_width = names.iter().map(|s| s.len()).max().unwrap_or(0);
    let col_width = max_width + MIN_GAP;
    let num_cols = (TERM_WIDTH / col_width).max(1);
    let num_rows = (names.len() + num_cols - 1) / num_cols;

    for row in 0..num_rows {
        for col in 0..num_cols {
            let idx = row * num_cols + col;
            if idx < names.len() {
                let name = &names[idx];
                if col < num_cols - 1 {
                    output.push_str(&format!("{:<width$}", name, width = col_width));
                } else {
                    output.push_str(name);
                }
            }
        }
        output.push('\n');
    }
}

fn ls_cmp(a: &str, b: &str) -> Ordering {
    let a_dot = a.starts_with('.');
    let b_dot = b.starts_with('.');

    match (a_dot, b_dot) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a.to_lowercase().cmp(&b.to_lowercase()),
    }
}

fn suffix_for<'a>(path: &'a Path, meta: &'a fs::Metadata) -> &'a str {
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
    let bits = [0o400, 0o200, 0o100, 0o040, 0o020, 0o010, 0o004, 0o002, 0o001];
    let chars = ['r', 'w', 'x'];

    for i in 0..9 {
        s.push(if mode & bits[i] != 0 {
            chars[i % 3]
        } else {
            '-'
        });
    }

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

    let mut display_name = name.to_string();
    if meta.file_type().is_symlink() {
        if let Ok(target_path) = fs::read_link(path) {
            display_name.push_str(&format!(" -> {}", target_path.to_string_lossy()));
        }
    }

    if !meta.file_type().is_symlink() {
        display_name.push_str(suffix_for(path, meta));
    }

    format!(
        "{}{} {:>2} {:<8} {:<8} {} {} {}",
        file_type, perms, nlink, user, group, size_or_dev, date_str, display_name
    )
}
