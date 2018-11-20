use std::io;
use std::process::Command;

/*
* Uses the installed git command to initialize a project repo.
*/
pub fn git_init(url: &str) {
    println!("Working...");

    // Initialize the current directory as a git repo
    let output = match Command::new("git").args(&["init"]).output() {
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
    let output = match Command::new("git").args(&["remote", "add", "origin", url]).output() {
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
pub fn git_add_and_commit(commit_message: String) {
    let mut message = String::new();

    if commit_message.is_empty() {
        // Get the commit message from the user to mark these changes with
        println!("Message to attach to these project changes:");

        io::stdin().read_line(&mut message)
            .expect("ERROR: Failed to read change message line from user");
    }
    else {
        message = commit_message;
    }

    // git add .
    let output = match Command::new("git").args(&["add", "."]).output() {
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
    let output = match Command::new("git").args(&["commit", "-m", &message]).output() {
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
    let output = match Command::new("git").args(&["push", "origin", "master"]).output() {
        Ok(out) => {
            println!("Changes pushed using git.");
            out
        },
        Err(e) => panic!("ERROR: Unable to push changes to remote git repository: {}", e)
    };

    if !output.stderr.is_empty() {
        panic!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
    }
}


/*
* Pulls latest updates from a component's git repo.
*/
pub fn git_pull() {
    let output = match Command::new("git").args(&["pull", "origin", "master"]).output() {
        Ok(out) => {
            println!("Pulled changes from component repository.");
            out
        },
        Err(e) => panic!("ERROR: Unable to pull changes from component repository: {}", e)
    };

    if !output.stderr.is_empty() {
        panic!("ERROR: {}", String::from_utf8_lossy(&output.stderr));
    }
}


/*
* Interface to the git command to download a component from a repo.
*/
pub fn git_clone(url: &str) {
    let output = match Command::new("git").args(&["clone", "--recursive", url]).output() {
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