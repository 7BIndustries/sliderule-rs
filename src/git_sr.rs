use std::process::Command;
use std::path::Path;
use git2::{Repository, AutotagOption, FetchOptions}; //RemoteCallbacks

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
    let mut output = super::SROutput { status: 0, stdout: Vec::new(), stderr: Vec::new() };

    // Maybe later we will support other branches
    let args = Args { arg_remote: Some(String::from("origin")) };

    // Open the local repo
    let repo = match Repository::open(target_dir) {
        Ok(repo) => repo,
        Err(e) => {
            output.status = 100;
            output.stderr.push(format!("Could not open local repository directory: {}", e));
            return output;
        }
    };

    // Try to find the remote reference so that we can use it to fetch the changes
    let remote = args.arg_remote.as_ref().map(|s| &s[..]).unwrap_or("origin");
    let mut remote = match repo.find_remote(remote) {
        Ok(rem) => rem,
        Err(_) => match repo.remote_anonymous(remote) {
            Ok(rem) => rem,
            Err(e) => {
                output.status = 101;
                output.stderr.push(format!("Could not find remote reference: {}", e));
                return output;
            }
        }
    };

    // Download any changes
    let mut fo = FetchOptions::new();
    match remote.download(&[], Some(&mut fo)) {
        //.expect("ERROR: Could not download from remote.");
        Ok(rem) => rem,
        Err(e) => {
            output.status = 102;
            output.stderr.push(format!("Could not download from remote: {}", e));
        }
    };

    // Disconnect the underlying connection to prevent from idling.
    remote.disconnect();

    // Update the references in the remote's namespace to point to the right commits
    match remote.update_tips(None, true, AutotagOption::Unspecified, None) {
        //.expect("ERROR: Could not update the references to point to the right commits.");
        Ok(rem) => rem,
        Err(e) => {
            output.status = 103;
            output.stderr.push(format!("Could not update the references to point to the right commits: {}", e));
        }
    };

    // Let the user know that we finished
    output.stdout.push(String::from("git pull complete"));

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