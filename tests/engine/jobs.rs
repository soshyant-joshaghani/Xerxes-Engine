//! Reading a job's output: colours stripped, and the address the game is served on found.

use xerxes_engine::editor::project::jobs::{JobStatus, preview_url, strip_ansi};

fn lines(text: &[&str]) -> Vec<String> {
    text.iter().map(|l| l.to_string()).collect()
}

#[test]
fn terminal_colours_are_removed() {
    assert_eq!(
        strip_ansi("\u{1b}[33mwarning\u{1b}[0m: slow"),
        "warning: slow"
    );
    assert_eq!(strip_ansi("plain"), "plain");
    assert_eq!(
        strip_ansi("\u{1b}[1;32m  Compiling\u{1b}[0m x"),
        "  Compiling x"
    );
    // A cut-off sequence does not hang or panic.
    assert_eq!(strip_ansi("a\u{1b}[3"), "a");
}

#[test]
fn the_pipelines_own_line_gives_the_address() {
    let log = lines(&[
        "[xerxes] my-game-5 on http://localhost:5200  (http://game.localhost with `backend run dev`)",
        "Serving at http://127.0.0.1:9999",
    ]);
    assert_eq!(preview_url(&log).as_deref(), Some("http://localhost:5200"));

    // Colours around it, and the newest announcement wins (a restart picks another port).
    let log = lines(&[
        "\u{1b}[32m[xerxes]\u{1b}[0m a on http://localhost:5200",
        "dev server proxies http://localhost:8000/api/",
        "[xerxes] a on http://localhost:5201",
    ]);
    assert_eq!(preview_url(&log).as_deref(), Some("http://localhost:5201"));
}

#[test]
fn other_addresses_and_half_lines_are_not_the_game() {
    assert_eq!(preview_url(&[]), None);
    assert_eq!(
        preview_url(&lines(&[
            "proxy http://localhost:8000/api/",
            "http://localhost:5200"
        ])),
        None,
        "only the pipeline's own [xerxes] line counts"
    );
    assert_eq!(
        preview_url(&lines(&["[xerxes] x on http://localhost:"])),
        None
    );
}

#[test]
fn a_job_status_reads_from_the_backends_json() {
    let json = r#"{"id":3,"args":["game","dev","my-game","web"],"status":"running","exit_code":null,"log":["[xerxes] my-game on http://localhost:5200"]}"#;
    let job: JobStatus = serde_json::from_str(json).unwrap();
    assert!(job.running() && job.is_dev());
    assert_eq!(job.title(), "game dev my-game web");
    assert_eq!(job.preview_url().as_deref(), Some("http://localhost:5200"));

    let done: JobStatus = serde_json::from_str(
        r#"{"id":4,"args":["game","build","x","web"],"status":"succeeded","exit_code":0,"log":[]}"#,
    )
    .unwrap();
    assert!(!done.running() && !done.is_dev());
    // Missing optional parts do not break reading.
    let bare: JobStatus = serde_json::from_str(r#"{"id":5,"status":"failed"}"#).unwrap();
    assert!(bare.log.is_empty() && bare.exit_code.is_none());
}

#[test]
fn a_finished_build_says_where_its_result_is() {
    use xerxes_engine::editor::project::jobs::output_path;
    // The order the pipeline prints them in: the build, then its Dockerfile. The build line wins.
    let web = lines(&[
        "[xerxes] x web build: dist/x/web/public (serve it as a static site)",
        "[xerxes] x: Dockerfile in dist/x/web (docker build -t x-web .)",
    ]);
    assert_eq!(output_path(&web).as_deref(), Some("dist/x/web/public"));
    // No build line: the newest line that names a path.
    let only = lines(&["[xerxes] x: Dockerfile in dist/x/web (docker build -t x-web .)"]);
    assert_eq!(output_path(&only).as_deref(), Some("dist/x/web"));
    let windows = lines(&["[xerxes] x desktop build: dist/x/windows/x.exe"]);
    assert_eq!(
        output_path(&windows).as_deref(),
        Some("dist/x/windows/x.exe")
    );
    // Colours, other lines and the pipeline's chatter are not results.
    let noisy = lines(&[
        "\u{1b}[32m[xerxes]\u{1b}[0m x web build: dist/x/web/public (done)",
        "error: expected dist/x/web",
        "compiling dist/other",
    ]);
    assert_eq!(output_path(&noisy).as_deref(), Some("dist/x/web/public"));
    assert_eq!(output_path(&lines(&["[xerxes] nothing here"])), None);
    assert_eq!(output_path(&lines(&["[xerxes] dist/"])), None);
    assert_eq!(output_path(&[]), None);
}

#[cfg(not(target_arch = "wasm32"))]
mod local {
    use std::process::Command;
    use std::time::{Duration, Instant};

    use xerxes_engine::editor::project::jobs::{JobRequest, local};

    fn request(command: &str, platform: &str, template: bool) -> JobRequest {
        JobRequest {
            command: command.into(),
            target: "my-game".into(),
            platform: platform.into(),
            template,
        }
    }

    #[test]
    fn the_pipeline_arguments_follow_the_command_line() {
        assert_eq!(
            local::args(&request("dev", "web", false)).unwrap(),
            ["game", "dev", "my-game", "web"]
        );
        assert_eq!(
            local::args(&request("publish", "windows", true)).unwrap(),
            ["template", "publish", "my-game", "windows"]
        );
        assert!(local::args(&request("rm", "web", false)).is_err());
        assert!(local::args(&request("dev", "ps5", false)).is_err());
    }

    #[test]
    fn a_job_collects_its_output_and_ends_with_its_exit_status() {
        let mut command = Command::new("rustc");
        command.arg("--version");
        let first = local::run(command, vec!["rustc".into(), "--version".into()]).unwrap();
        assert!(first.running() || first.status == "succeeded");

        let started = Instant::now();
        let done = loop {
            let now = local::status(first.id).unwrap();
            if !now.running() {
                break now;
            }
            assert!(
                started.elapsed() < Duration::from_secs(30),
                "the job never ended"
            );
            std::thread::sleep(Duration::from_millis(50));
        };
        assert_eq!(done.status, "succeeded");
        assert_eq!(done.exit_code, Some(0));
        assert!(
            done.log.iter().any(|l| l.starts_with("rustc ")),
            "{:?}",
            done.log
        );
        assert_eq!(done.title(), "rustc --version");
    }

    #[test]
    fn a_failing_job_is_failed_and_a_missing_program_is_an_error() {
        let mut command = Command::new("rustc");
        command.arg("--no-such-flag-xerxes");
        let job = local::run(command, vec!["rustc".into()]).unwrap();
        let started = Instant::now();
        let done = loop {
            let now = local::status(job.id).unwrap();
            if !now.running() {
                break now;
            }
            assert!(started.elapsed() < Duration::from_secs(30));
            std::thread::sleep(Duration::from_millis(50));
        };
        assert_eq!(done.status, "failed");
        assert_ne!(done.exit_code, Some(0));
        assert!(!done.log.is_empty(), "the error text is in the log");

        let err = local::run(Command::new("xerxes-no-such-program"), vec![]).unwrap_err();
        assert!(err.contains("cannot run"), "{err}");
        assert!(local::status(u64::MAX).is_err());
        assert!(local::stop(u64::MAX).is_err());
    }
}
