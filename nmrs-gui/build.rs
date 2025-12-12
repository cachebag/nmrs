use std::process::Command;

fn main() {
    let output = Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output();

    let hash = match output {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => {
            println!("cargo:warning=Unable to determine git hash, using 'unknown'");
            String::from("unknown")
        }
    };

    println!("cargo:rustc-env=GIT_HASH={}", hash);
}
