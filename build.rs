fn main() {
    println!("cargo:rerun-if-changed=assets/app-icon.ico");
    if std::env::var("CARGO_CFG_TARGET_OS").ok().as_deref() != Some("windows") {
        return;
    }
    if let Err(err) = embed_icon() {
        println!("cargo:warning=app icon resource not embedded: {err}");
    }
}

fn embed_icon() -> Result<(), Box<dyn std::error::Error>> {
    if let Some(dir) = find_rc_dir() {
        let path = std::env::var("PATH").unwrap_or_default();
        std::env::set_var("PATH", format!("{};{path}", dir.display()));
    }
    let mut res = winres::WindowsResource::new();
    res.set_icon("assets/app-icon.ico");
    res.compile()?;
    Ok(())
}

fn find_rc_dir() -> Option<std::path::PathBuf> {
    const ROOTS: &[&str] = &[
        r"C:\Program Files (x86)\Windows Kits\10\bin",
        r"C:\Program Files\Windows Kits\10\bin",
    ];
    for root in ROOTS {
        let Ok(entries) = std::fs::read_dir(root) else {
            continue;
        };
        let mut dirs: Vec<_> = entries.filter_map(|e| e.ok()).collect();
        dirs.sort_by_key(|e| e.file_name());
        for entry in dirs.into_iter().rev() {
            let rc = entry.path().join("x64").join("rc.exe");
            if rc.is_file() {
                return rc.parent().map(|p| p.to_path_buf());
            }
        }
    }
    None
}
