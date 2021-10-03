
use std::fs;
use std::process::Command;
use std::path::Path;

fn compile_shader<P: AsRef<Path>>(path: P) {
    let path = path.as_ref();
    let mut file_name = path.file_name().unwrap().to_os_string();
    file_name.push(".spv");
    let output = Command::new("glslc")
        .arg("-c")
        .arg(path)
        .arg("-o")
        .arg(Path::new("target/shaders").join(file_name))
        .output()
        .expect("failed to spawn glslc and get output");
    if !output.status.success() {
        panic!("glslc failed");
    }
}

fn main() {
    fs::create_dir_all("target/shaders").expect("failed to create target/shaders");
    compile_shader("src/shaders/fill.frag");
}
