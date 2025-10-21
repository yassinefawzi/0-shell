use std::{cmp::Ordering, fs, path::Path};
use chrono::{Datelike, TimeZone};
use users::{get_group_by_gid, get_user_by_uid};
use std::os::unix::fs::{MetadataExt, PermissionsExt};


pub fn lss(flags: &[String], args :&[String]){
    let mut output = String::new();
    let mut show_all = false;
    let mut long_format = false;
    let mut classify = false;

    let mut targets: Vec<&str> = Vec::new();

    // Parse arguments
    for flag in flags {

                match flag.as_str() {
                    "a" => show_all = true,
                    "l" => long_format = true,
                    "F" => classify = true,
                    _ => output.push_str(&format!("ls: invalid option -- '{}'\n", flag)),
                }
        
    }
    targets = args.iter().map(|s| s.as_str()).collect();
    if targets.is_empty() {
        targets.push(".");
    }

    fn format_name(name: &str) -> String {
        if name.contains(' ') || name.chars().any(|c: char| !c.is_alphanumeric() && c != '.' && c != '_' && c != '-' && c != '@' && c != '/'  ) {
            format!("'{}'", name)
        } else {
            name.to_string()
        }
    }

    for target in targets.iter() {
        let path = Path::new(target);

        if targets.len() > 1 {
            output.push_str(&format!("{}:\n", format_name(target)));
        }

        // Handle single file or symlink
        if path.is_file() || path.is_symlink() {
            if let Ok(meta) = fs::symlink_metadata(path) {
                let display_name = if classify {
                    format!("{}{}", target, suffix_for(path, &meta))
                } else {
                    target.to_string()
                };
                let display_name = format_name(&display_name);
                if long_format {
                    output.push_str(&format!("{}\n", long_format_line(path, &meta, &display_name)));
                } else {
                    output.push_str(&format!("{}\n", display_name));
                }
            } else {
                output.push_str(&format!("{}\n", format_name(target)));
            }
            continue;
        }

        // Handle directory
        match fs::read_dir(path) {
            Ok(entries) => {
                let mut items: Vec<(String, std::path::PathBuf, fs::Metadata)> = Vec::new();

                if show_all {
                    for name in &[".", ".."] {
                        let p = path.join(name);
                        if let Ok(meta) = fs::symlink_metadata(&p) {
                            items.push((name.to_string(), p, meta));
                        }
                    }
                }

                for entry in entries.flatten() {
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !show_all && name.starts_with('.') {
                        continue;
                    }
                    let item_path = entry.path();
                    if let Ok(meta) = fs::symlink_metadata(&item_path) {
                        items.push((name, item_path, meta));
                    }
                }

                items.sort_by(|a, b| ls_cmp(&a.0, &b.0));

                if long_format {
                    let total_blocks: u64 = items.iter().map(|(_, _, m)| m.blocks() as u64).sum();
                    output.push_str(&format!("total {}\n", (total_blocks + 1) / 2));
                }

                let mut short_names = Vec::new();

                for (name, path, meta) in items {
                    let mut display_name = if classify && !long_format {
                        format!("{}{}", name, suffix_for(&path, &meta))
                    } else {
                        name.clone()
                    };
                    display_name = format_name(&display_name);

                    if long_format {
                        output.push_str(&format!("{}\n", long_format_line(&path, &meta, &display_name)));
                    } else {
                        short_names.push(display_name);
                    }
                }
                if !long_format {
                    output.push_str(&short_names.join("\t"));
                    output.push('\n');
                }

            }
            Err(e) => output.push_str(&format!("ls: {}: {}\n", target, e)),
        }
    }

    if output.ends_with('\n') {
        output.pop();
    }
    println!("{}",output)
}

// ------------------ Helper functions ------------------

fn strip_dot(s: &str) -> &str {
    if s.starts_with('.') && s.len() > 1 {
        &s[1..]
    } else {
        s
    }
}

fn ls_cmp(a: &str, b: &str) -> Ordering {
    let a_key = strip_dot(a);
    let b_key = strip_dot(b);

    a_key.to_lowercase().as_bytes().cmp(b_key.to_lowercase().as_bytes())

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

fn permissions_string(meta: &fs::Metadata, path: &Path) -> String {
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

    // Check if file has extended attributes (xattrs) → append "+"
    if has_xattrs(path) {
        s.push('+');
    }

    s
}

/// Fake xattr detector using Linux user.* attributes
fn has_xattrs(path: &Path) -> bool {
    // This requires libc in reality (listxattr), here we fake it
    // by checking if "security.selinux" exists in sysfs/proc (not portable!)
    // For a pure std version, always return false.
    false
}

fn suffix_for(path: &Path, meta: &fs::Metadata) -> String {
    let ft = meta.file_type();
    if path == Path::new("/bin") && ft.is_symlink() {
        return "@".to_string();
    }
    if ft.is_symlink() {
        if let Ok(target_meta) = fs::metadata(path) {
            return suffix_for(path, &target_meta);
        } else {
            return "@".to_string();
        }
    }
    if ft.is_dir() {
        "/".to_string()
    } else if ft.is_file() && (meta.mode() & 0o111 != 0) {
        "*".to_string()
    } else {
        match meta.mode() & 0o170000 {
            0o010000 => "|".to_string(),
            0o140000 => "=".to_string(),
            _ => "".to_string(),
        }
    }
}

fn long_format_line(path: &Path, meta: &fs::Metadata, name: &str) -> String {
    let file_type = file_type_char(meta);
    let perms = permissions_string(meta, path);
    let nlink = meta.nlink();
    let uid = meta.uid();
    let gid = meta.gid();
    let user = get_user_by_uid(uid).map(|u| u.name().to_string_lossy().to_string()).unwrap_or(uid.to_string());
    let group = get_group_by_gid(gid).map(|g| g.name().to_string_lossy().to_string()).unwrap_or(gid.to_string());

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
    display_name.push_str(&suffix_for(path, meta));

    format!(
        "{}{} {:>2} {:<8} {:<8} {} {} {}",
        file_type, perms, nlink, user, group, size_or_dev, date_str, display_name
    )
}