pub fn configure() {
    println!("cargo:rerun-if-changed=build/main.rs");
    println!("cargo:rerun-if-env-changed=PROFILE");
    println!("cargo:rerun-if-env-changed=LIBSODIUM_PREFIX");
 
    // Use system-installed ZeroMQ instead of building from source
    println!("cargo:warning=Using system-installed ZeroMQ with libsodium support");
    
    // Let system-deps find the system zeromq installation
    if let Err(e) = system_deps::Config::new().probe() {
        eprintln!("Failed to find system zeromq: {}", e);
        std::process::exit(1);
    }
}
 
fn main() {
    configure()
}
 