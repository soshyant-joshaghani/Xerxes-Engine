//! `xerxes-new <templates-dir> <template> <games-dir> <name> [--engine-path <path>]`
//!
//! Creates `<games-dir>/<name>` from `<templates-dir>/<template>` (or, for `starter`, the engine's
//! built-in empty project) with the shared scaffold
//! (`xerxes_build::scaffold`), the same code the editor and the backend use. Without
//! `--engine-path` the game depends on the engine's Git repository. `xerxes-ctrl game new`
//! runs this inside the repo with `--engine-path ../../engine`.

use std::path::Path;
use std::process::ExitCode;

use xerxes_build::scaffold::{self, ENGINE_GIT, EngineSource};

fn folders(dir: &Path) -> Vec<String> {
    std::fs::read_dir(dir)
        .map(|entries| {
            entries
                .flatten()
                .filter(|e| e.path().is_dir())
                .map(|e| e.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default()
}

fn run(args: &[String]) -> Result<(), String> {
    let [templates_dir, template, games_dir, name, rest @ ..] = args else {
        return Err("usage: xerxes-new <templates-dir> <template> <games-dir> <name> [--engine-path <path>]".into());
    };
    let engine = match rest {
        [] => EngineSource::Git {
            url: ENGINE_GIT.into(),
            branch: "main".into(),
        },
        [flag, path] if flag == "--engine-path" => EngineSource::Path(path.clone()),
        _ => return Err(format!("unexpected arguments: {}", rest.join(" "))),
    };
    let (templates_dir, games_dir) = (Path::new(templates_dir), Path::new(games_dir));
    let templates = folders(templates_dir);
    let games: Vec<String> = folders(games_dir)
        .into_iter()
        .filter(|g| !g.starts_with("__"))
        .collect();
    let taken: Vec<&str> = games.iter().map(String::as_str).collect();
    let reserved: Vec<&str> = templates.iter().map(String::as_str).collect();
    if let Some(problem) = scaffold::name_problem(name, &taken, &reserved) {
        return Err(problem);
    }
    let files = if template == scaffold::STARTER {
        scaffold::starter_files()
    } else {
        let source = templates_dir.join(template);
        if !source.join("Cargo.toml").is_file() {
            return Err(format!(
                "no template `{template}` (have: {}, and the built-in {})",
                templates.join(", "),
                scaffold::STARTER
            ));
        }
        scaffold::read_dir(&source).map_err(|e| e.to_string())?
    };
    let files = scaffold::customize(files, template, name, &engine).map_err(|e| e.to_string())?;
    scaffold::write(&games_dir.join(name), &files).map_err(|e| e.to_string())?;
    println!(
        "created {} ({} files) from {template}",
        games_dir.join(name).display(),
        files.len()
    );
    Ok(())
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("error: {err}");
            ExitCode::FAILURE
        }
    }
}
