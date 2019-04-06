extern crate os_info;

use std::path::Path;
use std::process::Command;

struct Args {
    arg_remote: Option<String>,
}

/// Uses the installed git command to initialize a new component project repo.
///
/// `target_dir` must be a valid Sliderule component directory.
/// 'url' The URL of the remote repository to set as the origin for this git repository.
///
/// This module is primarily for sliderule-rs use, and direct use should be avoided in most situations.
pub fn git_init(target_dir: &Path, url: &str) -> super::SROutput {
    let mut output = super::SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    // Initialize the current directory as a git repo
    let stdoutput = match Command::new("git")
        .args(&["init"])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                output.status = 106;
                output
                    .stderr
                    .push(format!("ERROR: `git` was not found, please install: {}", e));
                return output;
            } else {
                output.status = 107;
                output
                    .stderr
                    .push(format!("ERROR: Could not initialize git repository: {}", e));
                return output;
            }
        }
    };
    // init success
    output
        .stderr
        .push(String::from("git repository initialized for project."));
    // init stderr
    if !output.stderr.is_empty() {
        output
            .stderr
            .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());
    }

    // Add the remote URL
    let stdoutput = match Command::new("git")
        .args(&["remote", "add", "origin", url])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            output.status = 108;
            output.stderr.push(format!(
                "ERROR: Unable to set remote URL for project: {}",
                e
            ));
            return output;
        }
    };
    // init success
    output.stdout.push(String::from(
        "Done initializing git repository for project.",
    ));
    // init stderr
    if !output.stderr.is_empty() {
        output
            .stderr
            .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());
    }

    output
}

/// Adds, commits and pushes any local component changes to the remote git repo.
///
/// `target_dir` must be a valid Sliderule component directory.
/// `message` commit message to attach to the changes when pushing to the remote repository.
///
/// This module is primarily for sliderule-rs use, and direct use should be avoided in most situations.
pub fn git_add_and_commit(target_dir: &Path, message: String) -> super::SROutput {
    let mut output = super::SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    // git add .
    let stdoutput = match Command::new("git")
        .args(&["add", "."])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            output.status = 103;
            output
                .stderr
                .push(format!("ERROR: Unable to stage changes using git: {}", e));
            return output;
        }
    };
    // Collect all of the other stdout entries
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());
    // Staging success
    output
        .stdout
        .push(String::from("Changes staged using git."));
    // Staging stderr
    output
        .stderr
        .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());

    let info = os_info::get();

    // git push will hang in some configurations on Windows if we don't disable the git sendpack.sideband option
    if info.os_type() == os_info::Type::Windows {
        let stdoutput = match Command::new("git")
            .args(&["config", "--local", "sendpack.sideband", "false"])
            .current_dir(target_dir)
            .output()
        {
            Ok(out) => out,
            Err(e) => {
                output.status = 109;
                output.stderr.push(format!(
                    "ERROR: Unable to disable sendpack.sideband git option: {}",
                    e
                ));
                return output;
            }
        };
        // Collect all of the other stdout entries
        output
            .stdout
            .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());
        // Staging stderr
        output
            .stderr
            .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());
    }

    // git commit -m [message]
    let stdoutput = match Command::new("git")
        .args(&["commit", "-m", &message])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            output.status = 104;
            output
                .stderr
                .push(format!("ERROR: Unable to commit changes using git: {}", e));
            return output;
        }
    };
    // Collect all of the other stdout entries
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());
    // Commit success
    output
        .stdout
        .push(String::from("Changes committed using git."));
    // Commit stderr
    output
        .stderr
        .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());

    // git push origin master
    let stdoutput = match Command::new("git")
        .args(&["push", "origin", "master"])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            output.status = 105;
            output.stderr.push(format!(
                "ERROR: Unable to push changes to remote git repository: {}",
                e
            ));
            return output;
        }
    };
    // Collect all of the other stdout entries
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());
    // Push success
    output
        .stdout
        .push(String::from("Changes pushed using git."));
    // Push stderr
    output
        .stderr
        .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());

    output
}

/// Pulls latest updates from a component's git repo.
///
/// `target_dir` must be a valid Sliderule component directory.
///
/// This module is primarily for sliderule-rs use, and direct use should be avoided in most situations.
pub fn git_pull(target_dir: &Path) -> super::SROutput {
    let mut output = super::SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    // Run the pull command
    let stdoutput = match Command::new("git")
        .args(&["pull", "origin", "master"])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            output.status = 100;
            output.stderr.push(format!(
                "ERROR: Pull from remote repository not successful: {}",
                e
            ));
            return output;
        }
    };

    // If we didn't get any output, the command is probably waiting on something
    if stdoutput.stdout.is_empty() {
        output.status = 101;
        output.stderr.push(format!(
            "ERROR: Pull failed, may be waiting for username/password or passphrase."
        ));
    }

    // Collect all of the other stdout entries
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());

    // If there were errors, make sure we collect them
    output
        .stderr
        .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());

    // If we have something other than a 0 exit status, report that
    if stdoutput.status.code().unwrap() != 0 {
        output.wrapped_status = stdoutput.status.code().unwrap();
    }

    output
}

/// Interface to the git command to download a component from a repo.
///
/// `target_dir` must be a valid Sliderule component directory.
/// 'url' The URL of the remote repository to clone (copy).
///
/// This module is primarily for sliderule-rs use, and direct use should be avoided in most situations.
pub fn git_clone(target_dir: &Path, url: &str) -> super::SROutput {
    let mut output = super::SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    let stdoutput = match Command::new("git")
        .args(&["clone", "--recursive", url])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            output.status = 102;
            output.stderr.push(format!(
                "ERROR: Unable to clone component repository: {}",
                e
            ));
            return output;
        }
    };

    // Collect all of the other stdout entries
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());

    // If there were errors, make sure we collect them
    output
        .stderr
        .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());

    // If we have something other than a 0 exit status, report that
    if stdoutput.status.code().unwrap() != 0 {
        output.wrapped_status = stdoutput.status.code().unwrap();
    }

    output
}

/// Changes the URL for the remote repository for the component.
///
/// `target_dir` must be a valid Sliderule component directory.
/// `url` URL to set for the remote repository.
///
/// This module is primarily for sliderule-rs use, and direct use should be avoided in most situations.
pub fn git_set_remote_url(target_dir: &Path, url: &str) -> super::SROutput {
    let mut output = super::SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    let stdoutput = match Command::new("git")
        .args(&["remote", "set-url", "origin", url])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            output.status = 110;
            output.stderr.push(format!(
                "ERROR: Unable to change the URL on the component repository: {}",
                e
            ));
            return output;
        }
    };

    // Collect all of the other stdout entries
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());

    // If there were errors, make sure we collect them
    output
        .stderr
        .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());

    // If we have something other than a 0 exit status, report that
    if stdoutput.status.code().unwrap() != 0 {
        output.wrapped_status = stdoutput.status.code().unwrap();
    }

    output
}

/// Runs the equivalent of `git status` on a component to get a listing of the high-level changes.
///
/// `target_dir` must be a valid Sliderule component directory.
///
/// This module is primarily for sliderule-rs use, and direct use should be avoided in most situations.
pub fn git_status(target_dir: &Path) -> super::SROutput {
    let mut output = super::SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    let stdoutput = match Command::new("git")
        .args(&["status"])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            output.status = 111;
            output.stderr.push(format!(
                "ERROR: Unable to change the URL on the component repository: {}",
                e
            ));
            return output;
        }
    };

    // Collect all of the other stdout entries
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());

    // If there were errors, make sure we collect them
    output
        .stderr
        .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());

    // If we have something other than a 0 exit status, report that
    if stdoutput.status.code().unwrap() != 0 {
        output.wrapped_status = stdoutput.status.code().unwrap();
    }

    return output;
}

/// Runs the equivalent of `git status` on a component to get the detailed changes per file.
///
/// `target_dir` must be a valid Sliderule component directory.
///
/// This module is primarily for sliderule-rs use, and direct use should be avoided in most situations.
pub fn git_diff(target_dir: &Path) -> super::SROutput {
    let mut output = super::SROutput {
        status: 0,
        wrapped_status: 0,
        stdout: Vec::new(),
        stderr: Vec::new(),
    };

    let stdoutput = match Command::new("git")
        .args(&["--no-pager", "diff"])
        .current_dir(target_dir)
        .output()
    {
        Ok(out) => out,
        Err(e) => {
            output.status = 112;
            output.stderr.push(format!(
                "ERROR: Unable to change the URL on the component repository: {}",
                e
            ));
            return output;
        }
    };

    // Collect all of the other stdout entries
    output
        .stdout
        .push(String::from_utf8_lossy(&stdoutput.stdout).to_string());

    // If there were errors, make sure we collect them
    output
        .stderr
        .push(String::from_utf8_lossy(&stdoutput.stderr).to_string());

    // If we have something other than a 0 exit status, report that
    if stdoutput.status.code().unwrap() != 0 {
        output.wrapped_status = stdoutput.status.code().unwrap();
    }

    return output;
}
