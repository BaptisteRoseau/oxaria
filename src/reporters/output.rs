//! Where each requested report goes, the up-front check that no two of them
//! (or a report and the checked file) share a destination, and the parallel
//! write of every report once the check is done.

use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::sync::Arc;

use super::{GithubReporter, GitlabReporter, JenkinsReporter, JunitReporter, Report, Reporter};
use crate::cli::CliConfig;
use crate::error::CheckerError;
use crate::page;

const STDOUT_ARG: &str = "-";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Destination {
    Stdout,
    File(PathBuf),
}

impl Destination {
    fn parse(arg: &str) -> Self {
        match arg {
            STDOUT_ARG => Destination::Stdout,
            path => Destination::File(PathBuf::from(path)),
        }
    }

    fn describe(&self) -> String {
        match self {
            Destination::Stdout => "stdout".to_string(),
            Destination::File(path) => path.display().to_string(),
        }
    }

    /// What two destinations are compared by: two spellings of one file
    /// (`r.json`, `./x/../r.json`, a symlinked directory) must collide.
    fn identity(&self) -> Option<PathBuf> {
        match self {
            Destination::Stdout => None,
            Destination::File(path) => Some(resolved(path)),
        }
    }
}

pub struct ReportTarget {
    pub flag: &'static str,
    pub reporter: Arc<dyn Reporter>,
    pub destination: Destination,
}

/// Builds one target per `--report-*` flag, failing on any destination
/// conflict so the caller can abort before spending time on the check.
pub fn targets_from(config: &CliConfig) -> Result<Vec<ReportTarget>, CheckerError> {
    let requested: [(&'static str, &Option<String>, Arc<dyn Reporter>); 4] = [
        (
            "--report-gitlab",
            &config.report_gitlab,
            Arc::new(GitlabReporter),
        ),
        (
            "--report-github",
            &config.report_github,
            Arc::new(GithubReporter),
        ),
        (
            "--report-jenkins",
            &config.report_jenkins,
            Arc::new(JenkinsReporter),
        ),
        (
            "--report-junit",
            &config.report_junit,
            Arc::new(JunitReporter),
        ),
    ];
    let targets: Vec<_> = requested
        .into_iter()
        .filter_map(|(flag, arg, reporter)| {
            arg.as_deref().map(|arg| ReportTarget {
                flag,
                reporter,
                destination: Destination::parse(arg),
            })
        })
        .collect();
    check_conflicts(&targets, &config.path_or_url)?;
    Ok(targets)
}

fn check_conflicts(targets: &[ReportTarget], path_or_url: &str) -> Result<(), CheckerError> {
    let identities: Vec<_> = targets.iter().map(|t| t.destination.identity()).collect();
    for (index, target) in targets.iter().enumerate() {
        if let Some(earlier) = (0..index).find(|&e| identities[e] == identities[index]) {
            return Err(CheckerError::ReportConflict {
                first: targets[earlier].flag,
                second: target.flag,
                destination: target.destination.describe(),
            });
        }
        if overwrites_input(identities[index].as_deref(), path_or_url) {
            return Err(CheckerError::ReportOverwritesInput {
                flag: target.flag,
                path: path_or_url.to_string(),
            });
        }
    }
    Ok(())
}

fn overwrites_input(identity: Option<&Path>, path_or_url: &str) -> bool {
    match identity {
        Some(identity) => {
            !page::is_url(path_or_url) && identity == resolved(Path::new(path_or_url))
        }
        None => false,
    }
}

/// Reports usually don't exist yet, so only their parent directory can be
/// canonicalized; the rest is resolved lexically.
fn resolved(path: &Path) -> PathBuf {
    let absolute = lexically_normalized(&std::path::absolute(path).unwrap_or(path.to_path_buf()));
    if let Ok(canonical) = absolute.canonicalize() {
        return canonical;
    }
    match (absolute.parent(), absolute.file_name()) {
        (Some(parent), Some(name)) => match parent.canonicalize() {
            Ok(parent) => parent.join(name),
            Err(_) => absolute,
        },
        _ => absolute,
    }
}

fn lexically_normalized(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                normalized.pop();
            }
            other => normalized.push(other),
        }
    }
    normalized
}

/// Writes every report concurrently, and keeps going after a failure so each
/// broken destination is reported, not just the first one.
pub async fn write_all(
    report: Arc<Report>,
    targets: Vec<ReportTarget>,
) -> Result<(), Vec<CheckerError>> {
    let tasks: Vec<_> = targets
        .into_iter()
        .map(|target| tokio::spawn(write_one(Arc::clone(&report), target)))
        .collect();

    let mut errors = Vec::new();
    for task in tasks {
        match task.await {
            Ok(Ok(())) => {}
            Ok(Err(err)) => errors.push(err),
            Err(join_error) => errors.push(CheckerError::ReportTask(join_error)),
        }
    }
    match errors.is_empty() {
        true => Ok(()),
        false => Err(errors),
    }
}

async fn write_one(report: Arc<Report>, target: ReportTarget) -> Result<(), CheckerError> {
    let content = target.reporter.render(&report);
    let result = match &target.destination {
        Destination::Stdout => write_stdout(&content),
        Destination::File(path) => write_file(path, &content).await,
    };
    result.map_err(|source| CheckerError::ReportWrite {
        flag: target.flag,
        destination: target.destination.describe(),
        source,
    })
}

/// One locked write, so the report can't interleave with other stdout output.
fn write_stdout(content: &str) -> std::io::Result<()> {
    let mut stdout = std::io::stdout().lock();
    stdout.write_all(content.as_bytes())?;
    stdout.flush()
}

async fn write_file(path: &Path, content: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent().filter(|p| !p.as_os_str().is_empty()) {
        tokio::fs::create_dir_all(parent).await?;
    }
    tokio::fs::write(path, content).await
}

#[cfg(test)]
mod tests {
    use clap::Parser;
    use rstest::rstest;

    use super::*;
    use crate::reporters::model::tests::error;

    fn config(args: &[&str]) -> CliConfig {
        CliConfig::parse_from(std::iter::once("wcag-checker").chain(args.iter().copied()))
    }

    fn flags_and_destinations(targets: &[ReportTarget]) -> Vec<(&str, Destination)> {
        targets
            .iter()
            .map(|t| (t.flag, t.destination.clone()))
            .collect()
    }

    #[test]
    fn no_report_flag_means_no_target() {
        assert!(targets_from(&config(&["a.html"])).unwrap().is_empty());
    }

    #[test]
    fn dash_means_stdout() {
        let targets = targets_from(&config(&["a.html", "--report-github", "-"])).unwrap();
        assert_eq!(
            flags_and_destinations(&targets),
            vec![("--report-github", Destination::Stdout)]
        );
    }

    #[test]
    fn every_flag_gets_its_own_target() {
        let targets = targets_from(&config(&[
            "a.html",
            "--report-gitlab",
            "gl.json",
            "--report-github",
            "-",
            "--report-jenkins",
            "reports/jk.json",
            "--report-junit",
            "junit.xml",
        ]))
        .unwrap();
        assert_eq!(
            flags_and_destinations(&targets),
            vec![
                ("--report-gitlab", Destination::File("gl.json".into())),
                ("--report-github", Destination::Stdout),
                (
                    "--report-jenkins",
                    Destination::File("reports/jk.json".into())
                ),
                ("--report-junit", Destination::File("junit.xml".into())),
            ]
        );
    }

    #[rstest]
    #[case("r.json", "r.json")]
    #[case("r.json", "./r.json")]
    #[case("r.json", "sub/../r.json")]
    #[case("-", "-")]
    fn two_reports_on_one_destination_conflict(#[case] first: &str, #[case] second: &str) {
        let result = targets_from(&config(&[
            "a.html",
            "--report-gitlab",
            first,
            "--report-junit",
            second,
        ]));
        assert!(
            matches!(
                result,
                Err(CheckerError::ReportConflict {
                    first: "--report-gitlab",
                    second: "--report-junit",
                    ..
                })
            ),
            "expected a conflict for {first} vs {second}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn symlinked_directories_conflict() {
        let dir = tempfile::tempdir().unwrap();
        let real = dir.path().join("real");
        let link = dir.path().join("link");
        std::fs::create_dir(&real).unwrap();
        std::os::unix::fs::symlink(&real, &link).unwrap();
        let result = targets_from(&config(&[
            "a.html",
            "--report-gitlab",
            real.join("r.json").to_str().unwrap(),
            "--report-junit",
            link.join("r.json").to_str().unwrap(),
        ]));
        assert!(matches!(result, Err(CheckerError::ReportConflict { .. })));
    }

    #[test]
    fn report_overwriting_the_input_file_conflicts() {
        let result = targets_from(&config(&["./page.html", "--report-junit", "page.html"]));
        assert!(matches!(
            result,
            Err(CheckerError::ReportOverwritesInput {
                flag: "--report-junit",
                ..
            })
        ));
    }

    #[test]
    fn a_url_input_never_conflicts_with_a_file() {
        let args = ["https://example.com/r.json", "--report-gitlab", "r.json"];
        assert!(targets_from(&config(&args)).is_ok());
    }

    #[test]
    fn distinct_files_do_not_conflict() {
        let args = [
            "a.html",
            "--report-gitlab",
            "a.json",
            "--report-jenkins",
            "b.json",
        ];
        assert!(targets_from(&config(&args)).is_ok());
    }

    #[tokio::test]
    async fn write_all_writes_every_report_creating_directories() {
        let dir = tempfile::tempdir().unwrap();
        let gitlab = dir.path().join("nested/gl.json");
        let junit = dir.path().join("junit.xml");
        let targets = targets_from(&config(&[
            "a.html",
            "--report-gitlab",
            gitlab.to_str().unwrap(),
            "--report-junit",
            junit.to_str().unwrap(),
        ]))
        .unwrap();
        let report = Arc::new(Report::new("a.html", &[error("H57")]));

        write_all(Arc::clone(&report), targets).await.unwrap();

        assert_eq!(
            std::fs::read_to_string(gitlab).unwrap(),
            GitlabReporter.render(&report)
        );
        assert_eq!(
            std::fs::read_to_string(junit).unwrap(),
            JunitReporter.render(&report)
        );
    }

    #[tokio::test]
    async fn write_all_reports_every_failure() {
        let dir = tempfile::tempdir().unwrap();
        let blocker = dir.path().join("file");
        std::fs::write(&blocker, "").unwrap();
        let targets = targets_from(&config(&[
            "a.html",
            "--report-gitlab",
            blocker.join("gl.json").to_str().unwrap(),
            "--report-junit",
            blocker.join("junit.xml").to_str().unwrap(),
        ]))
        .unwrap();

        let errors = write_all(Arc::new(Report::new("a.html", &[])), targets)
            .await
            .unwrap_err();

        assert_eq!(errors.len(), 2);
    }
}
