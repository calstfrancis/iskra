//! Ported verbatim from Zerkalo's `git_sync.rs` (no per-app constants to
//! change here — see `github_auth.rs`/`secret_store.rs` for those). Iskra's
//! single-repo model (`~/Documents/Iskra/`, no multi-remote backup UI) only
//! calls a subset of this API today; the rest stays available for parity
//! with Zerkalo and any future backup-remote feature.
#![allow(dead_code)]

use std::path::Path;
use std::process::Command;

use chrono::Local;

pub fn in_flatpak() -> bool {
    std::path::Path::new("/.flatpak-info").exists()
}

/// Returns a `Command` for a host binary, using `flatpak-spawn --host` when
/// running inside a flatpak sandbox so the binary is found on the host.
pub fn host_command(bin: &str) -> Command {
    if in_flatpak() {
        let mut cmd = Command::new("flatpak-spawn");
        cmd.arg("--host").arg(bin);
        cmd
    } else {
        Command::new(bin)
    }
}

/// Returns a `Command` pre-loaded with `git -C <repo>`, using
/// `flatpak-spawn --host git` when running inside a flatpak sandbox.
pub(crate) fn git_cmd(repo_path: &Path) -> Command {
    let mut cmd = if in_flatpak() {
        let mut cmd = Command::new("flatpak-spawn");
        cmd.args(["--host", "git", "-C", path_str(repo_path)]);
        cmd
    } else {
        let mut cmd = Command::new("git");
        cmd.args(["-C", path_str(repo_path)]);
        cmd
    };
    // Force English output so the substring matches in is_auth_error() and the
    // "nothing to commit" check below are reliable regardless of the user's locale.
    cmd.env("LANG", "C").env("LC_ALL", "C");
    cmd
}

// ── Public types ─────────────────────────────────────────────────────────────

pub struct SyncResult {
    pub committed: bool,
    /// True if at least one remote was pushed successfully.
    pub pushed: bool,
    pub commit_message: String,
    /// Fatal error (add or commit failed before any push).
    pub error: Option<String>,
    /// Non-fatal: per-remote push failures — "(remote_name) reason".
    pub push_errors: Vec<String>,
    /// True if any push error looks like an authentication failure.
    pub auth_failed: bool,
}

// ── Query helpers ─────────────────────────────────────────────────────────────

/// Returns the git repository root for the given directory, or None if not in a git repo.
pub fn git_repo_root(dir: &Path) -> Option<std::path::PathBuf> {
    let out = git_cmd(dir)
        .args(["rev-parse", "--show-toplevel"])
        .output()
        .ok()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if !s.is_empty() { Some(std::path::PathBuf::from(s)) } else { None }
    } else {
        None
    }
}

/// Returns true if the repo has at least one remote configured.
pub fn has_remote(repo_path: &Path) -> bool {
    git_cmd(repo_path)
        .arg("remote")
        .output()
        .map(|out| !out.stdout.trim_ascii().is_empty())
        .unwrap_or(false)
}

/// Returns the names of all configured remotes.
pub fn list_remotes(repo_path: &Path) -> Vec<String> {
    git_cmd(repo_path)
        .arg("remote")
        .output()
        .map(|out| {
            String::from_utf8_lossy(&out.stdout)
                .lines()
                .map(|l| l.trim().to_string())
                .filter(|l| !l.is_empty())
                .collect()
        })
        .unwrap_or_default()
}

/// Returns the push URL for a named remote.
pub fn get_remote_url(repo_path: &Path, name: &str) -> Option<String> {
    let out = git_cmd(repo_path)
        .args(["remote", "get-url", name])
        .output()
        .ok()?;
    if out.status.success() {
        let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
        if s.is_empty() { None } else { Some(s) }
    } else {
        None
    }
}

/// Add (or update) a remote named "backup". Removes any existing "backup" first.
/// If `target` is a local path (starts with `/`, `~`, `./`, or `../`), a bare
/// git repository is initialised there automatically so the path is ready to
/// receive pushes.
pub fn add_backup_remote(repo_path: &Path, target: &str) -> Result<(), String> {
    add_named_remote(repo_path, "backup", target)
}

/// Add (or update) a remote with `name`. Removes any existing remote with that
/// name first. Local paths get a bare repository initialised automatically.
/// The remote is pushed to on every `sync()` call alongside all other remotes.
pub fn add_named_remote(repo_path: &Path, name: &str, url: &str) -> Result<(), String> {
    let resolved = if is_local_path(url) {
        let expanded = shellexpand::tilde(url).into_owned();
        ensure_bare_repo(Path::new(&expanded))?;
        expanded
    } else {
        url.to_string()
    };
    let _ = run_git(repo_path, &["remote", "remove", name]);
    run_git(repo_path, &["remote", "add", name, &resolved])
}

/// Remove a named remote.
pub fn remove_remote(repo_path: &Path, name: &str) -> Result<(), String> {
    run_git(repo_path, &["remote", "remove", name])
}

/// Return all configured remotes except "origin", paired with their push URL.
/// These are the backup / secondary remotes that `sync()` also pushes to.
pub fn list_backup_remotes(repo_path: &Path) -> Vec<(String, String)> {
    list_remotes(repo_path)
        .into_iter()
        .filter(|n| n != "origin")
        .filter_map(|name| {
            let url = get_remote_url(repo_path, &name)?;
            Some((name, url))
        })
        .collect()
}

/// Returns true when the string looks like a filesystem path rather than a git URL.
pub fn is_local_path(s: &str) -> bool {
    s.starts_with('/') || s.starts_with('~') || s.starts_with("./") || s.starts_with("../")
}

/// Ensures `path` contains a bare git repository, creating one if needed.
fn ensure_bare_repo(path: &Path) -> Result<(), String> {
    if path.join("HEAD").exists() {
        return Ok(());
    }
    std::fs::create_dir_all(path).map_err(|e| e.to_string())?;
    run_git(path, &["init", "--bare"])
}

/// Returns the name of the current branch (falls back to "main").
pub fn current_branch(repo_path: &Path) -> String {
    git_cmd(repo_path)
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .output()
        .map(|out| String::from_utf8_lossy(&out.stdout).trim().to_string())
        .unwrap_or_else(|_| "main".to_string())
}

/// Returns display names of files changed since the last commit.
pub fn changed_files(repo_path: &Path) -> Vec<String> {
    let Ok(out) = git_cmd(repo_path)
        .args(["status", "--porcelain"])
        .output()
    else {
        return Vec::new();
    };

    if !out.status.success() {
        return Vec::new();
    }

    let mut names: Vec<String> = Vec::new();
    for line in String::from_utf8_lossy(&out.stdout).lines() {
        if line.len() < 4 {
            continue;
        }
        let entry = &line[3..];
        let filename = entry.split(" -> ").last().unwrap_or(entry).trim();
        let basename = Path::new(filename)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or(filename)
            .to_string();
        if !names.contains(&basename) {
            names.push(basename);
        }
    }
    names
}

pub struct FileHistoryEntry {
    pub commit: String,
    pub date: chrono::DateTime<Local>,
    pub message: String,
}

/// Returns the commit history for a single file (newest first), relative to
/// `repo_path`. Used by the in-app history browser to list past versions of
/// the open sermon without shelling out again per row.
pub fn file_history(repo_path: &Path, file_path: &Path) -> Vec<FileHistoryEntry> {
    let Ok(rel) = file_path.strip_prefix(repo_path) else {
        return Vec::new();
    };
    let Ok(out) = git_cmd(repo_path)
        .args(["log", "--follow", "--format=%H%x1f%aI%x1f%s", "--"])
        .arg(rel)
        .output()
    else {
        return Vec::new();
    };
    if !out.status.success() {
        return Vec::new();
    }
    String::from_utf8_lossy(&out.stdout)
        .lines()
        .filter_map(|line| {
            let mut parts = line.splitn(3, '\u{1f}');
            let commit = parts.next()?.to_string();
            let date = parts.next()?;
            let message = parts.next().unwrap_or("").to_string();
            let date = chrono::DateTime::parse_from_rfc3339(date)
                .ok()?
                .with_timezone(&Local);
            Some(FileHistoryEntry { commit, date, message })
        })
        .collect()
}

/// Returns the file's contents as of the given commit, relative to
/// `repo_path`.
pub fn show_file_at_commit(repo_path: &Path, file_path: &Path, commit: &str) -> Option<String> {
    let rel = file_path.strip_prefix(repo_path).ok()?;
    let spec = format!("{commit}:{}", path_str(rel));
    let out = git_cmd(repo_path).args(["show", &spec]).output().ok()?;
    if !out.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&out.stdout).to_string())
}

/// Build a human-readable commit message from the changed file list.
pub fn craft_message(changed: &[String]) -> String {
    let ts = Local::now().format("%Y-%m-%d %H:%M").to_string();
    match changed.len() {
        0 => format!("Auto-save: {ts}"),
        1 => format!("Edited {}: {ts}", changed[0]),
        _ => {
            let shown: Vec<&str> = changed.iter().take(5).map(String::as_str).collect();
            let suffix = if changed.len() > 5 {
                format!(" (+{})", changed.len() - 5)
            } else {
                String::new()
            };
            format!("Edits to {}{}\n\n{ts}", shown.join(", "), suffix)
        }
    }
}

// ── Write operations ─────────────────────────────────────────────────────────

/// Add a remote named "origin".
pub fn add_remote(repo_path: &Path, url: &str) -> Result<(), String> {
    run_git(repo_path, &["remote", "add", "origin", url])
}

/// Stage everything, commit with an auto-crafted message, pull from each remote
/// (rebase), then push to every configured remote.
///
/// `github_token` is injected into HTTPS GitHub remote URLs for authentication.
/// `pushed` is true if at least one remote succeeded.
pub fn sync(repo_path: &Path, github_token: Option<&str>) -> SyncResult {
    let changed = changed_files(repo_path);
    let msg = craft_message(&changed);

    if let Err(e) = run_git(repo_path, &["add", "."]) {
        return SyncResult {
            committed: false, pushed: false, commit_message: msg,
            error: Some(format!("git add: {e}")),
            push_errors: Vec::new(), auth_failed: false,
        };
    }

    let committed = match git_cmd(repo_path)
        .args(["commit", "-m", &msg])
        .output()
    {
        Err(e) => return SyncResult {
            committed: false, pushed: false, commit_message: msg,
            error: Some(format!("git commit: {e}")),
            push_errors: Vec::new(), auth_failed: false,
        },
        Ok(out) if !out.status.success() => {
            let text = lossy_combined(&out);
            if text.contains("nothing to commit") { false } else {
                return SyncResult {
                    committed: false, pushed: false, commit_message: msg,
                    error: Some(text), push_errors: Vec::new(), auth_failed: false,
                };
            }
        }
        Ok(_) => true,
    };

    let remotes = list_remotes(repo_path);
    let branch = current_branch(repo_path);
    let mut pushed = false;
    let mut push_errors: Vec<String> = Vec::new();
    let mut auth_failed = false;

    for remote in &remotes {
        let auth_args: Vec<String> = match github_token {
            Some(tok) if !tok.is_empty() => {
                match get_remote_url(repo_path, remote) {
                    Some(url) if is_github_https(&url) => github_auth_args(tok),
                    _ => Vec::new(),
                }
            }
            _ => Vec::new(),
        };

        // Pull --rebase before push so diverged histories are handled.
        if let Ok(pull_out) = git_cmd(repo_path)
            .args(auth_args.clone())
            .args(["pull", "--rebase", remote.as_str(), &branch])
            .output()
        {
            if !pull_out.status.success() {
                let msg = lossy_combined(&pull_out);
                if is_auth_error(&msg) {
                    auth_failed = true;
                }
                // `rebase --abort` is only meaningful if the pull actually
                // got as far as starting a rebase (a real conflict). Pull
                // can also fail earlier — auth, network, no such remote
                // branch — in which case there's nothing to abort, and
                // running it anyway produces its own "No rebase in
                // progress" error that used to get misreported as "the
                // repository may be in mid-rebase state," alarming the user
                // over a problem that didn't exist while burying the real
                // cause (e.g. an expired GitHub token).
                match git_cmd(repo_path).args(["rebase", "--abort"]).output() {
                    Ok(a) if !a.status.success() => {
                        let abort_msg = lossy_combined(&a);
                        if abort_msg.contains("No rebase in progress") {
                            push_errors.push(format!("({remote}) Couldn't pull latest changes: {msg}"));
                        } else {
                            push_errors.push(format!(
                                "({remote}) Pull failed and rebase --abort also failed: {abort_msg}. \
                                 Repository may be in mid-rebase state — run 'git rebase --abort' manually."
                            ));
                        }
                    }
                    Err(e) => {
                        push_errors.push(format!(
                            "({remote}) Pull failed and could not run rebase --abort: {e}. \
                             Repository may be in mid-rebase state — run 'git rebase --abort' manually."
                        ));
                    }
                    Ok(_) => {
                        push_errors.push(format!("({remote}) Couldn't pull latest changes: {msg}"));
                    }
                }
                continue;
            }
        }

        match git_cmd(repo_path)
            .args(auth_args.clone())
            .args(["push", "-u", remote.as_str(), &branch])
            .output() {
            Err(e) => push_errors.push(format!("({remote}) {e}")),
            Ok(o) if !o.status.success() => {
                let msg = lossy_combined(&o);
                if is_auth_error(&msg) { auth_failed = true; }
                push_errors.push(format!("({remote}) {msg}"));
            }
            Ok(_) => pushed = true,
        }
    }

    SyncResult { committed, pushed, commit_message: msg, error: None, push_errors, auth_failed }
}

/// Whether `url` is an `https://github.com/...` remote — the only case the
/// stored OAuth token is authorized for. Never send it to any other host.
fn is_github_https(url: &str) -> bool {
    url.starts_with("https://github.com/")
}

/// Builds `-c http.<url>.extraHeader=...` args that authenticate as `token`,
/// scoped to `https://github.com/` only. Passed as git config rather than
/// embedded in the remote URL so the token never appears in argv (visible via
/// `ps`/`/proc/<pid>/cmdline` for the life of the process) or in `git remote -v`.
fn github_auth_args(token: &str) -> Vec<String> {
    let encoded = base64_encode(format!("x-access-token:{token}").as_bytes());
    vec![
        "-c".to_string(),
        format!("http.https://github.com/.extraHeader=AUTHORIZATION: basic {encoded}"),
    ]
}

fn base64_encode(input: &[u8]) -> String {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(input.len().div_ceil(3) * 4);
    for chunk in input.chunks(3) {
        let b0 = chunk[0];
        let b1 = *chunk.get(1).unwrap_or(&0);
        let b2 = *chunk.get(2).unwrap_or(&0);
        let n = ((b0 as u32) << 16) | ((b1 as u32) << 8) | (b2 as u32);
        out.push(ALPHABET[(n >> 18 & 0x3F) as usize] as char);
        out.push(ALPHABET[(n >> 12 & 0x3F) as usize] as char);
        out.push(if chunk.len() > 1 { ALPHABET[(n >> 6 & 0x3F) as usize] as char } else { '=' });
        out.push(if chunk.len() > 2 { ALPHABET[(n & 0x3F) as usize] as char } else { '=' });
    }
    out
}

fn is_auth_error(msg: &str) -> bool {
    msg.contains("Authentication failed")
        || msg.contains("403")
        || msg.contains("401")
        || msg.contains("could not read Username")
        || msg.contains("remote: Invalid username")
}

// ── Internals ─────────────────────────────────────────────────────────────────

fn path_str(p: &Path) -> &str {
    p.to_str().unwrap_or(".")
}

fn run_git(repo_path: &Path, args: &[&str]) -> Result<(), String> {
    let out = git_cmd(repo_path)
        .args(args)
        .output()
        .map_err(|e| e.to_string())?;
    if out.status.success() {
        Ok(())
    } else {
        Err(lossy_combined(&out))
    }
}

/// Concatenates stdout and stderr rather than picking one — git spreads
/// diagnostics for the same failure across both (e.g. a rebase conflict's
/// actual `CONFLICT (content): ...` marker lands on stdout while the
/// surrounding "resolve manually / run --abort" hints go to stderr), so
/// preferring only one silently drops information the other holds.
fn lossy_combined(out: &std::process::Output) -> String {
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    match (stdout.is_empty(), stderr.is_empty()) {
        (true, true) => String::new(),
        (false, true) => stdout,
        (true, false) => stderr,
        (false, false) => format!("{stdout}\n{stderr}"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Real end-to-end reproduction of a rebase conflict during `sync()`,
    /// using actual `git` subprocesses against temp repos (no network/auth
    /// involved — a bare local repo stands in for "GitHub"). Guards against
    /// the exact bug this test was written for: a pull failure used to
    /// unconditionally run `rebase --abort`, and if that itself failed for
    /// an unrelated reason, the user saw "repository may be in mid-rebase
    /// state" even when the repo was already clean.
    #[test]
    fn sync_reports_conflict_and_leaves_repo_clean() {
        let tmp = tempfile::tempdir().unwrap();
        // `-b main` on both the bare repo and repo_a pins the branch name so
        // the bare repo's HEAD resolves correctly once pushed to, and
        // repo_b's clone below checks out `main` rather than an unborn
        // default branch.
        let bare = tmp.path().join("bare.git");
        run_git(tmp.path(), &["init", "--bare", "-b", "main", bare.to_str().unwrap()]).unwrap();

        // First repo: commit "one", push to the bare remote.
        let repo_a = tmp.path().join("a");
        std::fs::create_dir_all(&repo_a).unwrap();
        run_git(&repo_a, &["init", "-b", "main"]).unwrap();
        run_git(&repo_a, &["remote", "add", "origin", bare.to_str().unwrap()]).unwrap();
        set_test_identity(&repo_a);
        std::fs::write(repo_a.join("sermon.toml"), "title = \"one\"\n").unwrap();
        run_git(&repo_a, &["add", "."]).unwrap();
        run_git(&repo_a, &["commit", "-m", "one"]).unwrap();
        run_git(&repo_a, &["push", "-u", "origin", "main"]).unwrap();

        // Second clone of the same remote, diverges with a conflicting edit
        // to the same line, and pushes first — so repo_a's pull below hits
        // a real conflict on the same file.
        let repo_b = tmp.path().join("b");
        run_git(tmp.path(), &["clone", bare.to_str().unwrap(), repo_b.to_str().unwrap()]).unwrap();
        set_test_identity(&repo_b);
        std::fs::write(repo_b.join("sermon.toml"), "title = \"two\"\n").unwrap();
        run_git(&repo_b, &["add", "."]).unwrap();
        run_git(&repo_b, &["commit", "-m", "two"]).unwrap();
        run_git(&repo_b, &["push", "origin", "main"]).unwrap();

        // Back in repo_a, make a conflicting local change and sync — this
        // must fail to pull (real CONFLICT), but leave the repo usable.
        std::fs::write(repo_a.join("sermon.toml"), "title = \"three\"\n").unwrap();
        let result = sync(&repo_a, None);

        assert!(!result.pushed, "a genuine conflict must not report success");
        let detail = result.push_errors.join("\n");
        assert!(detail.contains("CONFLICT"), "expected a real conflict marker, got: {detail}");
        assert!(
            !detail.contains("may be in mid-rebase state"),
            "abort succeeded, so the repo is clean — this message must not appear: {detail}"
        );

        // The repo must not be left mid-rebase.
        assert!(!repo_a.join(".git/rebase-merge").exists());
        assert!(!repo_a.join(".git/rebase-apply").exists());
        let status = git_cmd(&repo_a).arg("status").output().unwrap();
        let status_text = String::from_utf8_lossy(&status.stdout);
        assert!(
            !status_text.contains("rebase in progress"),
            "repo left in a mid-rebase state: {status_text}"
        );
    }

    fn set_test_identity(path: &Path) {
        run_git(path, &["config", "user.email", "test@example.com"]).unwrap();
        run_git(path, &["config", "user.name", "Test"]).unwrap();
    }

    #[test]
    fn file_history_lists_newest_first_and_show_file_at_commit_reads_old_content() {
        let tmp = tempfile::tempdir().unwrap();
        let repo = tmp.path().join("repo");
        std::fs::create_dir_all(&repo).unwrap();
        run_git(&repo, &["init", "-b", "main"]).unwrap();
        set_test_identity(&repo);

        let file = repo.join("sermon.toml");
        std::fs::write(&file, "title = \"one\"\n").unwrap();
        run_git(&repo, &["add", "."]).unwrap();
        run_git(&repo, &["commit", "-m", "first"]).unwrap();

        std::fs::write(&file, "title = \"two\"\n").unwrap();
        run_git(&repo, &["add", "."]).unwrap();
        run_git(&repo, &["commit", "-m", "second"]).unwrap();

        let history = file_history(&repo, &file);
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].message, "second");
        assert_eq!(history[1].message, "first");

        let old_content = show_file_at_commit(&repo, &file, &history[1].commit).unwrap();
        assert_eq!(old_content, "title = \"one\"\n");
    }

    #[test]
    fn is_local_path_recognizes_absolute_and_relative_paths() {
        assert!(is_local_path("/home/user/repo"));
        assert!(is_local_path("~/repo"));
        assert!(is_local_path("./repo"));
        assert!(is_local_path("../repo"));
    }

    #[test]
    fn is_local_path_rejects_urls() {
        assert!(!is_local_path("https://github.com/foo/bar.git"));
        assert!(!is_local_path("git@github.com:foo/bar.git"));
    }

    #[test]
    fn craft_message_no_changes() {
        let msg = craft_message(&[]);
        assert!(msg.starts_with("Auto-save: "));
    }

    #[test]
    fn craft_message_single_file() {
        let msg = craft_message(&["main.typ".to_string()]);
        assert!(msg.starts_with("Edited main.typ: "));
    }

    #[test]
    fn craft_message_multiple_files_lists_up_to_five() {
        let files: Vec<String> = (1..=7).map(|i| format!("f{i}.typ")).collect();
        let msg = craft_message(&files);
        assert!(msg.starts_with("Edits to f1.typ, f2.typ, f3.typ, f4.typ, f5.typ (+2)"), "got: {msg}");
    }

    #[test]
    fn craft_message_exactly_five_files_no_suffix() {
        let files: Vec<String> = (1..=5).map(|i| format!("f{i}.typ")).collect();
        let msg = craft_message(&files);
        assert!(msg.starts_with("Edits to f1.typ, f2.typ, f3.typ, f4.typ, f5.typ\n"), "got: {msg}");
        assert!(!msg.contains('+'));
    }

    #[test]
    fn is_github_https_matches_only_github_com() {
        assert!(is_github_https("https://github.com/user/repo.git"));
        assert!(!is_github_https("https://example.com/repo.git"));
        assert!(!is_github_https("git@github.com:user/repo.git"));
    }

    #[test]
    fn github_auth_args_scopes_header_to_github_and_never_includes_raw_token() {
        let args = github_auth_args("abc123");
        assert_eq!(args[0], "-c");
        assert!(args[1].starts_with("http.https://github.com/.extraHeader=AUTHORIZATION: basic "));
        assert!(!args[1].contains("abc123"), "raw token must not appear in argv: {args:?}");
    }

    #[test]
    fn base64_encode_matches_known_vectors() {
        assert_eq!(base64_encode(b"x-access-token:abc123"), "eC1hY2Nlc3MtdG9rZW46YWJjMTIz");
        assert_eq!(base64_encode(b""), "");
        assert_eq!(base64_encode(b"a"), "YQ==");
        assert_eq!(base64_encode(b"ab"), "YWI=");
        assert_eq!(base64_encode(b"abc"), "YWJj");
    }

    #[test]
    fn is_auth_error_detects_common_auth_failures() {
        assert!(is_auth_error("remote: Authentication failed"));
        assert!(is_auth_error("fatal: could not read Username for 'https://...'"));
        assert!(is_auth_error("received 403 Forbidden"));
    }

    #[test]
    fn is_auth_error_false_for_unrelated_errors() {
        assert!(!is_auth_error("fatal: not a git repository"));
    }
}
