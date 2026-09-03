use std::{
    env, fs,
    io::{self, IsTerminal, Write},
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

#[derive(Debug, PartialEq, Eq)]
enum InstallMode {
    Cargo {
        root: Option<PathBuf>,
    },
    Nix {
        flakes: PathBuf,
        refresh: Option<bool>,
    },
}

fn usage() {
    eprintln!(
        "usage:\n  kallyup list\n  kallyup install <minimal|developer|full> [--root DIR]\n  kallyup install <minimal|developer|full> --nix --flakes DIR [--refresh-flakes|--no-refresh-flakes]\n\nprofiles:\n  minimal   Kalcite CLI and Kally\n  developer minimal plus Kalcite LSP\n  full      developer plus Kalcite Editor\n\nNix mode clones each selected flake into DIR and installs a profile at DIR/profile."
    );
}

fn option_value(arguments: &[String], index: &mut usize, option: &str) -> Result<PathBuf, String> {
    *index += 1;
    arguments
        .get(*index)
        .map(PathBuf::from)
        .ok_or_else(|| format!("{option} requires a directory"))
}

fn install_mode(arguments: &[String]) -> Result<InstallMode, String> {
    let mut root = None;
    let mut flakes = None;
    let mut nix = false;
    let mut refresh = None;
    let mut index = 3;

    while index < arguments.len() {
        match arguments[index].as_str() {
            "--root" => root = Some(option_value(arguments, &mut index, "--root")?),
            "--flakes" => flakes = Some(option_value(arguments, &mut index, "--flakes")?),
            "--nix" => nix = true,
            "--refresh-flakes" => refresh = Some(true),
            "--no-refresh-flakes" => refresh = Some(false),
            option => return Err(format!("unknown install option {option}")),
        }
        index += 1;
    }

    if nix {
        if root.is_some() {
            return Err(
                "--root cannot be combined with --nix; use --flakes DIR instead".to_owned(),
            );
        }
        let flakes = flakes.ok_or_else(|| "--nix requires --flakes DIR".to_owned())?;
        return Ok(InstallMode::Nix { flakes, refresh });
    }
    if flakes.is_some() || refresh.is_some() {
        return Err("--flakes and flake refresh options require --nix".to_owned());
    }
    Ok(InstallMode::Cargo { root })
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

fn run(program: &str, arguments: &[String]) -> Result<(), String> {
    let status = Command::new(program)
        .args(arguments)
        .status()
        .map_err(|error| format!("could not run {program}: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!("{program} exited unsuccessfully"))
    }
}

fn prepare_flake(directory: &PathBuf, repository: &str, refresh: bool) -> Result<(), String> {
    if !directory.exists() {
        if let Some(parent) = directory
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            fs::create_dir_all(parent)
                .map_err(|error| format!("could not create {}: {error}", parent.display()))?;
        }
        println!("cloning {repository} into {}...", directory.display());
        run(
            "git",
            &[
                "clone".to_owned(),
                "--branch".to_owned(),
                "main".to_owned(),
                "--depth".to_owned(),
                "1".to_owned(),
                repository.to_owned(),
                directory.display().to_string(),
            ],
        )?;
    }
    if !directory.join("flake.nix").is_file() {
        return Err(format!(
            "{} exists but is not a Kalcite flake checkout",
            directory.display()
        ));
    }
    if refresh {
        println!("refreshing flake inputs in {}...", directory.display());
        run(
            "nix",
            &[
                "flake".to_owned(),
                "update".to_owned(),
                "--flake".to_owned(),
                directory.display().to_string(),
            ],
        )?;
    }
    Ok(())
}

fn ask_to_refresh(refresh: Option<bool>) -> Result<bool, String> {
    if let Some(refresh) = refresh {
        return Ok(refresh);
    }
    if !io::stdin().is_terminal() {
        println!(
            "non-interactive session: keeping existing flake inputs (use --refresh-flakes to update them)"
        );
        return Ok(false);
    }
    print!("Refresh flake inputs before installing? [y/N] ");
    io::stdout()
        .flush()
        .map_err(|error| format!("could not prompt for flake refresh: {error}"))?;
    let mut answer = String::new();
    io::stdin()
        .read_line(&mut answer)
        .map_err(|error| format!("could not read flake refresh answer: {error}"))?;
    Ok(matches!(
        answer.trim().to_ascii_lowercase().as_str(),
        "y" | "yes" | "o" | "oui"
    ))
}

fn install_nix(
    flakes: PathBuf,
    refresh: Option<bool>,
    selected: &[(u32, &str, &str)],
) -> Result<(), String> {
    if Command::new("nix").arg("--version").status().is_err() {
        return Err("Nix is required for --nix. Install Nix, then run Kallyup again.".to_owned());
    }
    let refresh = ask_to_refresh(refresh)?;
    let profile = flakes.join("profile");
    for (_, repository, package) in selected {
        let checkout = flakes.join(repository);
        let git_url = format!("https://github.com/Kalcite-Engine/{repository}.git");
        prepare_flake(&checkout, &git_url, refresh)?;
        println!("installing {package} with Nix...");
        run(
            "nix",
            &[
                "profile".to_owned(),
                "install".to_owned(),
                "--profile".to_owned(),
                profile.display().to_string(),
                format!("{}#{package}", checkout.display()),
            ],
        )?;
    }
    println!(
        "Nix installation complete. Add {} to PATH if needed.",
        profile.join("bin").display()
    );
    Ok(())
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
            let mode = match install_mode(&args) {
                Ok(mode) => mode,
                Err(error) => {
                    eprintln!("{error}");
                    usage();
                    return ExitCode::FAILURE;
                }
            };
            let components = klc_core::kallyup_profile_components(profile);
            let selected = [
                (CORE, "kalcite", "kalcite"),
                (KALLY, "kally", "kally"),
                (LSP, "kalcite-lsp", "kalcite-lsp"),
                (EDITOR, "kalcite-editor", "kalcite-editor"),
            ];
            let enabled: Vec<_> = selected
                .into_iter()
                .filter(|(flag, _, _)| klc_core::kallyup_component_enabled(components, *flag))
                .collect();
            match mode {
                InstallMode::Cargo { root } => {
                    for (_, repository, bin) in enabled {
                        let package = if repository == "kalcite" {
                            "kalcite-cli"
                        } else {
                            repository
                        };
                        println!("installing {bin}...");
                        if let Err(error) = install(root.clone(), package, bin) {
                            eprintln!("{error}");
                            return ExitCode::FAILURE;
                        }
                    }
                    println!(
                        "installation complete. Add the selected Cargo bin directory to PATH if needed."
                    );
                }
                InstallMode::Nix { flakes, refresh } => {
                    if let Err(error) = install_nix(flakes, refresh, &enabled) {
                        eprintln!("{error}");
                        return ExitCode::FAILURE;
                    }
                }
            }
            ExitCode::SUCCESS
        }
        _ => {
            usage();
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nix_installation_directory_and_refresh_choice() {
        let arguments = vec![
            "kallyup".to_owned(),
            "install".to_owned(),
            "developer".to_owned(),
            "--nix".to_owned(),
            "--flakes".to_owned(),
            "/tmp/kalcite-flakes".to_owned(),
            "--refresh-flakes".to_owned(),
        ];
        assert_eq!(
            install_mode(&arguments),
            Ok(InstallMode::Nix {
                flakes: PathBuf::from("/tmp/kalcite-flakes"),
                refresh: Some(true)
            })
        );
    }

    #[test]
    fn requires_a_flake_directory_for_nix() {
        let arguments = vec![
            "kallyup".to_owned(),
            "install".to_owned(),
            "minimal".to_owned(),
            "--nix".to_owned(),
        ];
        assert!(
            install_mode(&arguments)
                .unwrap_err()
                .contains("--flakes DIR")
        );
    }
}
