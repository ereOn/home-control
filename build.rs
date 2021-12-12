use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=frontend/*");

    Command::new("npm")
        .args(["install"])
        .current_dir("frontend")
        .status()
        .expect("failed to run npm install");

    Command::new("npm")
        .args(["run", "build"])
        .current_dir("frontend")
        .status()
        .expect("failed to run npm run build");
}
