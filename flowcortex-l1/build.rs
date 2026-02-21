fn main() -> Result<(), Box<dyn std::error::Error>> {
    // use vendored protoc so that build works in environments without system
    // compiler installed (e.g., container-based CI).
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    tonic_build::configure()
        .protoc_path(protoc)
        .build_server(true)
        .compile(&["proto/l1.proto"], &["proto"])?;
    Ok(())
}
