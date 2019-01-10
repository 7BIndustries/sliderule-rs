use std::process::Command;
use std::path::Path;

#[derive(Deserialize)]
struct Args {
    arg_remote: Option<String>,
}

/*
* Uses the installed git command to initialize a project repo.
*/
pub fn git_init(target_dir: &Path, url: &str) {
    println!("Working...");

    // Initialize the current directory as a git repo
    let output = match Command::new("git").args(&["init"]).current_dir(target_dir).output() {
        Ok(out) => {
            println!("git repository initialized for project.");
            out
        },
        Err(e) => {
            if let std::io::ErrorKind::NotFound = e.kind() {
                panic!("ERROR: `git` was not found, please install");
            } else {
                panic!("ERROR: Could not initialize git repository: {}", e);
            }
        }
    };

    // Let the user know if something went wrong
    if !output.stderr.is_empty() {
        panic!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
    }

    // Add the remote URL
    let output = match Command::new("git").args(&["remote", "add", "origin", url]).current_dir(target_dir).output() {
        Ok(out) => {
            println!("Done initializing git repository for project.");
            out
        },
        Err(e) => panic!("ERROR: Unable to set remote URL for project: {}", e)
    };

    // Let the user know if something went wrong
    if !output.stderr.is_empty() {
        panic!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
    }
}


/*
* Adds, commits and pushes any changes to the remote git repo.
*/
pub fn git_add_and_commit(target_dir: &Path, message: String) {
    // git add .
    let output = match Command::new("git").args(&["add", "."]).current_dir(target_dir).output() {
        Ok(out) => {
            println!("Changes staged using git.");
            out
        },
        Err(e) => panic!("ERROR: Unable to stage changes using git: {}", e)
    };

    if !output.stderr.is_empty() {
        panic!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
    }

    // git commit -m [message]
    let output = match Command::new("git").args(&["commit", "-m", &message]).current_dir(target_dir).output() {
        Ok(out) => {
            println!("Changes committed using git.");
            out
        },
        Err(e) => panic!("ERROR: Unable to push changes to remote git repository: {}", e)
    };

    if !output.stderr.is_empty() {
        panic!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
    }

    // git push origin master
    let output = match Command::new("git").args(&["push", "origin", "master"]).current_dir(target_dir).output() {
        Ok(out) => {
            println!("Changes pushed using git.");
            out
        },
        Err(e) => panic!("ERROR: Unable to push changes to remote git repository: {}", e)
    };

    if !output.stderr.is_empty() {
        // panic!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
        println!("{}", String::from_utf8_lossy(&output.stderr));
    }
}

/*
* Pulls latest updates from a component's git repo.
*/
pub fn git_pull(target_dir: &Path) -> super::SROutput {
    let mut output = super::SROutput { status: 0, wrapped_status: 0, stdout: Vec::new(), stderr: Vec::new() };

    // Run the pull command
    let stdoutput = match Command::new("git").args(&["pull", "origin", "master"]).current_dir(target_dir).output() {
        Ok(out) => {
            out
        },
        Err(e) => {
            output.status = 100;
            output.stderr.push(format!("ERROR: Pull from remote repository not successful: {}", e));
            return output;
        }
    };

    // If we didn't get any output, the command is probably waiting on something
    if stdoutput.stdout.is_empty() {
        output.status = 100;
        output.stderr.push(format!("ERROR: Pull failed, may be waiting for username/password or passphrase."));
    }

    // Collect all of the other stdout entries
    output.stdout.push(String::from_utf8_lossy(&stdoutput.stdout).to_string());

    // If there were errors, make sure we collect them
    if !stdoutput.stderr.is_empty() {
        output.stderr.push(String::from_utf8_lossy(&stdoutput.stderr).to_string());
    }

    // If we have something other than a 0 exit status, report that
    if stdoutput.status.code().unwrap() != 0 {
        output.wrapped_status = stdoutput.status.code().unwrap();
    }

    output
}


/*
* Interface to the git command to download a component from a repo.
*/
pub fn git_clone(target_dir: &Path, url: &str) {
    let output = match Command::new("git").args(&["clone", "--recursive", url]).current_dir(target_dir).output() {
        Ok(out) => {
            println!("Successfully cloned component repository.");
            out
        },
        Err(e) => panic!("ERROR: Unable to clone component repository: {}", e)
    };

    if !output.stderr.is_empty() {
        panic!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
    }
}