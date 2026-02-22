fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Prefer system protoc when available; fall back to vendored binary.
    if std::env::var_os("PROTOC").is_none() {
        let protoc = protoc_bin_vendored::protoc_bin_path()?;
        // Safety: build scripts run in a controlled environment; setting an
        // env var here is safe and scoped to the build process.
        unsafe {
            std::env::set_var("PROTOC", protoc);
        }
    }
    tonic_build::configure()
        .build_server(true)
        .compile(&["proto/l1.proto"], &["proto"])?;
    Ok(())
}
