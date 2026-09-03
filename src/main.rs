use std::{
    env,
    path::PathBuf,
    process::{Command, ExitCode},
};

#[allow(dead_code, unused_mut, unused_parens)]
mod klc_core {
    include!(concat!(env!("OUT_DIR"), "/kallyup_core.rs"));
}

const CORE: u32 = 1;
const KALLY: u32 = 2;
const LSP: u32 = 4;
const EDITOR: u32 = 8;

fn usage() {
    eprintln!(
        "usage:\n  kallyup list\n  kallyup install <minimal|developer|full> [--root DIR]\n\nprofiles:\n  minimal   Kalcite CLI and Kally\n  developer minimal plus Kalcite LSP\n  full      developer plus Kalcite Editor"
    );
}

fn profile(value: &str) -> Option<u32> {
    match value {
        "minimal" => Some(1),
        "developer" => Some(2),
        "full" => Some(3),
        _ => None,
    }
}

fn install(root: Option<PathBuf>, package: &str, bin: &str) -> Result<(), String> {
    let mut command = Command::new("cargo");
    command.args(["install", "--git", "https://github.com/Kalcite-Engine/"]);
    let repository = match package {
        "kalcite-cli" => "kalcite.git",
        "kally" => "kally.git",
        "kalcite-lsp" => "kalcite-lsp.git",
        "kalcite-editor" => "kalcite-editor.git",
        _ => return Err(format!("unknown package {package}")),
    };
    command
        .arg(repository)
        .args(["--branch", "main", "--package", package, "--bin", bin]);
    if let Some(root) = root {
        command.arg("--root").arg(root);
    }
    let status = command
        .status()
        .map_err(|error| format!("could not run cargo: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("cargo could not install {package}"))
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    match args.get(1).map(String::as_str) {
        Some("list") => {
            println!(
                "minimal: kalcite, kally\ndeveloper: kalcite, kally, kalcite-lsp\nfull: kalcite, kally, kalcite-lsp, kalcite-editor"
            );
            ExitCode::SUCCESS
        }
        Some("install") => {
            let Some(profile) = args.get(2).and_then(|value| profile(value)) else {
                usage();
                return ExitCode::FAILURE;
            };
            let root = args
                .windows(2)
                .find(|pair| pair[0] == "--root")
                .map(|pair| PathBuf::from(&pair[1]));
            let components = klc_core::kallyup_profile_components(profile);
            let selected = [
                (CORE, "kalcite-cli", "kalcite"),
                (KALLY, "kally", "kally"),
                (LSP, "kalcite-lsp", "kalcite-lsp"),
                (EDITOR, "kalcite-editor", "kalcite-editor"),
            ];
            for (flag, package, bin) in selected {
                if klc_core::kallyup_component_enabled(components, flag) {
                    println!("installing {bin}...");
                    if let Err(error) = install(root.clone(), package, bin) {
                        eprintln!("{error}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            println!(
                "installation complete. Add the selected Cargo bin directory to PATH if needed."
            );
            ExitCode::SUCCESS
        }
        _ => {
            usage();
            ExitCode::FAILURE
        }
    }
}
