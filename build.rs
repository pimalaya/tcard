#[cfg(feature = "cli")]
use pimalaya_cli::build::{features_env, target_envs};

#[cfg(feature = "cli")]
fn main() {
    features_env(include_str!("./Cargo.toml"));
    target_envs();
    git_envs();
}

// NOTE: pimalaya_cli::build::git_envs panics on a repo with no commits
// yet; this resilient variant forwards GIT_DESCRIBE / GIT_REV from the
// environment (release builds set them), falls back to a read-only git
// query, then to "unknown".
#[cfg(feature = "cli")]
fn git_envs() {
    println!(
        "cargo::rustc-env=GIT_DESCRIBE={}",
        git_env(
            "GIT_DESCRIBE",
            &["describe", "--always", "--tags", "--dirty"]
        )
    );
    println!(
        "cargo::rustc-env=GIT_REV={}",
        git_env("GIT_REV", &["rev-parse", "HEAD"])
    );
}

#[cfg(feature = "cli")]
fn git_env(key: &str, args: &[&str]) -> String {
    let from_env = std::env::var(key).ok().filter(|value| !value.is_empty());

    from_env
        .or_else(|| {
            let output = std::process::Command::new("git").args(args).output().ok()?;

            output
                .status
                .success()
                .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_else(|| String::from("unknown"))
}

#[cfg(not(feature = "cli"))]
fn main() {}
