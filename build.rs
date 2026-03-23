fn main() {
    // DuckDB on Windows requires rstrtmgr.lib (Restart Manager API)
    if cfg!(target_os = "windows") {
        println!("cargo:rustc-link-lib=rstrtmgr");
    }
}
