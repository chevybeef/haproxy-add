use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::process::exit;

/// Very small, opinionated helper for a very specific config shape:
///
///   frontend Internal
///       bind *:443 ssl crt ...
///       mode http
///       acl is_X hdr(host) -i X.example.com
///       ...
///       use_backend X_backend if is_X
///       ...
///   backend X_backend
///       mode http
///       server X 1.2.3.4:5678 [ssl verify none]
///
/// It does NOT parse haproxy.cfg properly - it just finds the last
/// "    acl is_" line and the last "    use_backend " line inside the
/// frontend block, and inserts new lines right after them. Then it
/// appends a new backend block at the end of the file.
///
/// It never touches the original file: it writes "<path>.new" next to
/// it so you can diff before applying.
fn prompt(label: &str) -> String {
    print!("{label}: ");
    io::stdout().flush().ok();
    let mut buf = String::new();
    io::stdin()
        .read_line(&mut buf)
        .expect("failed to read stdin");
    buf.trim().to_string()
}

fn prompt_yn(label: &str, default_no: bool) -> bool {
    let hint = if default_no { "y/N" } else { "Y/n" };
    let answer = prompt(&format!("{label} [{hint}]"));
    if answer.is_empty() {
        return !default_no;
    }
    matches!(answer.to_lowercase().as_str(), "y" | "yes")
}

/// Turn "My Cool App" into "my_cool_app"
fn slugify(s: &str) -> String {
    s.trim()
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

const PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PKG_NAME: &str = env!("CARGO_PKG_NAME");
const GIT_SHA: &str = env!("VERGEN_GIT_SHA");
const GIT_DIRTY: &str = env!("VERGEN_GIT_DIRTY");

fn main() {
    let now = chrono::Local::now();
    let dirty = if GIT_DIRTY == "true" { "-dirty" } else { "" };
    println!(
        "{PKG_NAME} v{PKG_VERSION} ({GIT_SHA}{dirty}) at {} on {}",
        now.format("%Y-%m-%d %H:%M:%S"),
        std::env::consts::OS,
    );
    let args: Vec<String> = env::args().collect();
    let cfg_path = if args.len() > 1 {
        args[1].clone()
    } else {
        prompt("Path to haproxy.cfg")
    };

    let cfg_path = Path::new(&cfg_path);
    let original = match fs::read_to_string(cfg_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Couldn't read {}: {e}", cfg_path.display());
            exit(1);
        }
    };

    println!("--- New entry ---");
    let service_raw = prompt("Service name (e.g. myapp)");
    let service = slugify(&service_raw);
    if service.is_empty() {
        eprintln!("Service name can't be empty");
        exit(1);
    }

    let default_host = format!("{service}.chevybeef.com");
    let host = prompt(&format!("Hostname [{default_host}]"));
    let host = if host.is_empty() { default_host } else { host };

    let addr = prompt("Backend address (ip:port)");
    if addr.is_empty() {
        eprintln!("Backend address can't be empty");
        exit(1);
    }

    let use_ssl = prompt_yn("Backend speaks SSL (ssl verify none)?", true);

    let acl_name = format!("is_{service}");
    let backend_name = format!("{service}_backend");

    let acl_line = format!("    acl {acl_name} hdr(host) -i {host}");
    let use_backend_line = format!("    use_backend {backend_name} if {acl_name}");

    let server_line = if use_ssl {
        format!("    server {service} {addr} ssl verify none")
    } else {
        format!("    server {service} {addr}")
    };

    let backend_block = format!("\nbackend {backend_name}\n    mode http\n{server_line}\n");

    let lines: Vec<&str> = original.lines().collect();

    // Find last "    acl is_" line and last "    use_backend " line.
    let last_acl_idx = lines
        .iter()
        .rposition(|l| l.trim_start().starts_with("acl is_"));
    let last_use_backend_idx = lines
        .iter()
        .rposition(|l| l.trim_start().starts_with("use_backend "));

    let (Some(acl_idx), Some(ub_idx)) = (last_acl_idx, last_use_backend_idx) else {
        eprintln!(
            "Couldn't find existing 'acl is_' / 'use_backend' lines to anchor on - \
             this tool only knows how to extend a config that already has at least one of each."
        );
        exit(1);
    };

    // Build the new file line by line, inserting after the anchor lines.
    let mut out_lines: Vec<String> = Vec::with_capacity(lines.len() + 3);
    for (i, line) in lines.iter().enumerate() {
        out_lines.push(line.to_string());
        if i == acl_idx {
            out_lines.push(acl_line.clone());
        }
        if i == ub_idx {
            out_lines.push(use_backend_line.clone());
        }
    }

    let mut new_contents = out_lines.join("\n");
    new_contents.push('\n');
    new_contents.push_str(&backend_block);

    let out_path = env::current_dir()
        .expect("Failed to get current path")
        .join(cfg_path.file_name().expect("couldn't get file name"));

    if let Err(e) = fs::write(&out_path, &new_contents) {
        eprintln!("Failed to write {}: {e}", out_path.display());
        exit(1);
    }

    println!("\nWrote {}", out_path.display());
    println!(
        "Review with: diff {} {}",
        cfg_path.display(),
        out_path.display()
    );
    println!("Then apply manually and reload haproxy yourself.");
}
