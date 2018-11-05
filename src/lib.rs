pub mod sliderule {
    extern crate liquid;
    extern crate walkdir;

    use std::io;
    use std::env;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::io::prelude::*;

    use git_sr;
    use npm_sr;

    /*
    * Create a new Sliderule component or convert an existing project to being a Sliderule project.
    */
    pub fn create_component(name: &String, src_license: &String, docs_license: &String) {
        let mut source_license = String::new();
        let mut doc_license = String::new();

        // Check to see if the current directory is a component
        let is_component = Path::new("components").exists() && Path::new("bom_data.yaml").exists();

        // The path can either lead to a top level component (project), or a component nested within a project
        let mut component_dir = Path::new("components").join(name);

        // This is a top level component (project)
        if !is_component {
            component_dir = Path::new(name).to_path_buf();
        }
        else {
            source_license = get_license(true);
            doc_license = get_license(false);
        }

        // Create a directory for our component
        match fs::create_dir(&component_dir) {
            Ok(dir) => dir,
            Err(error) => {
                panic!("ERROR: Could not create dist directory: {:?}", error);
            }
        };

        // Make a new directory in components, cd into it, and then run the rest of this code
        match env::set_current_dir(&component_dir) {
            Ok(dir) => dir,
            Err(e) => {
                panic!("ERROR: Could not change into components directory: {}", e);
            }
        };

        // Create the components directory, if needed
        if !Path::new("components").exists() {
            match fs::create_dir("components") {
                Ok(dir) => dir,
                Err(error) => {
                    panic!("ERROR: Could not create components directory: {:?}", error);
                }
            };
        }
        else {
            println!("components directory already exists, using existing directory.");
        }

        // Create the dist directory, if needed
        if !Path::new("dist").exists() {
            match fs::create_dir("dist") {
                Ok(dir) => dir,
                Err(error) => {
                    panic!("ERROR: Could not create dist directory: {:?}", error);
                }
            };
        }
        else {
            println!("dist directory already exists, using existing directory.");
        }

        // Create the docs directory, if needed
        if !Path::new("docs").exists() {
            match fs::create_dir("docs") {
                Ok(dir) => dir,
                Err(error) => {
                    panic!("ERROR: Could not create docs directory: {:?}", error);
                }
            };
        }
        else {
            println!("docs directory already exists, using existing directory.");
        }

        //Create the source directory, if needed
        if !Path::new("source").exists() {
            match fs::create_dir("source") {
                Ok(dir) => dir,
                Err(error) => {
                    panic!("ERROR: Could not create source directory: {:?}", error);
                }
            };
        }
        else {
            println!("source directory already exists, using existing directory.");
        }

        // Generate the template readme file
        generate_readme(&name);

        // Generate bom_data.yaml
        generate_bom(&name);

        // If we're creating a top-level component we want ot ask for a license directly, otherwise get it from the parent component
        if !is_component {
            // Ask the user for their license choice for the source of this component if they haven't specified it on the command line
            if src_license.is_empty() {
                println!("Please choose a source license for this component.");
                println!("For a list of available licenses see https://spdx.org/licenses/");
                println!("Choice [Unlicense]:");
                io::stdin().read_line(&mut source_license)
                    .expect("ERROR: Failed to read name or license from user.");

                // If the user didn't choose a license, default to The Unlicense
                source_license = source_license.trim().to_string();
                if source_license.is_empty() {
                    source_license = String::from("Unlicense");
                }
            }
            else {
                source_license = src_license.to_string();
            }

            if docs_license.is_empty() {
                // Ask the user for their license choice for the documentation of this component
                println!("Please choose a documentation license for this component.");
                println!("For a list of available licenses see https://spdx.org/licenses/");
                println!("Choice [CC-BY-4.0]:");
                io::stdin().read_line(&mut doc_license)
                    .expect("ERROR: Failed to read name or license from user.");

                // If the user didn't choose a license, default to The Unlicense
                doc_license = doc_license.trim().to_string();
                if doc_license.is_empty() {
                    doc_license = String::from("CC-BY-4.0");
                }
            }
            else {
                doc_license = docs_license.to_string();
            }
        }

        // Generate package.json, if needed
        generate_package_json(&name, &source_license);

        // Generate the .sr file that provides extra information about this component
        let mut is_top = false;
        if !is_component {
            is_top = true;
        }
        generate_dot_file(is_top, &source_license, &doc_license);

        println!("Finished setting up component.");
    }


    /*
     * Uploads any changes to the project to the remote repository.
     */
    pub fn project_upload() {
        let mut url = String::new();

        // Make sure this project has already been initialized as a repository
        if !Path::new(".git").exists() {
            println!("This project has not been initialized with a repository yet. Enter a URL of an existing repository to upload this component to:");

            io::stdin().read_line(&mut url)
                .expect("ERROR: Failed to read name or URL from user.");

            // Initialize the git repository and set the remote URL to push to
            git_sr::git_init(url.trim());

            // Generate gitignore file so that we don't commit and push things we shouldn't be
            generate_gitignore();
        }
        
        // Add all changes, commit and push
        git_sr::git_add_and_commit();
    }


    /*
    * Converts a local component into a remote component, asking for a remote repo to push it to.
    */
    pub fn refactor(name: &str) {
        let mut url = String::new();
        
        println!("Please enter the URL of an existing repository to upload the component to:");

        io::stdin().read_line(&mut url)
                .expect("ERROR: Failed to read name or URL from user.");

        let orig_dir = get_cwd();
        let component_dir = Path::new("components").join(name);

        if component_dir.exists() {
            // We need to be in the component's directory before running the next commands
            match env::set_current_dir(&component_dir) {
                Ok(dir) => dir,
                Err(e) => {
                    panic!("ERROR: Could not change into components directory: {}", e);
                }
            };

            // Set the directory up as a git repo and then push the changes to the remote
            git_sr::git_init(&url.trim());
            git_sr::git_add_and_commit();

            // Change back up to the original, top level directory
            match env::set_current_dir(&orig_dir) {
                Ok(dir) => dir,
                Err(e) => {
                    panic!("ERROR: Could not change into original parent directory: {}", e);
                }
            };

            // Remove the local component and then install it from the remote using npm
            remove(&name);
            npm_sr::npm_install(&url.trim());
        }
        else {
            panic!("ERROR: The component does not exist in the components directory.");
        }
    }


    /*
    * Removes a component from the project structure.
    */
    pub fn remove(name: &str) {
        // let mut answer = String::new();

        // TODO: The user has to spell the component name out, so maybe that's enough of a safety check?
        // println!("Type Y/y and hit enter to continue removing this component: {}", name);

        // io::stdin().read_line(&mut answer)
        //     .expect("ERROR: Failed to read answer from user.");

        // Make sure that the answer was really yes on removal of the component
        // if &answer.trim().to_uppercase() != "Y" {
        //     println!("Aborting component removal.");

        //     return;
        // }

        let component_dir = Path::new("components").join(name);

        // If the component exists as a subdirectory of components delete the directory directly otherwise use npm to remove it.
        if component_dir.exists() {
            println!("Deleting component directory.");

            // Step through every file and directory in the path to be deleted and make sure that none are read-only
            for entry in walkdir::WalkDir::new(&component_dir) {
                let entry = match entry {
                    Ok(ent) => ent,
                    Err(e) => panic!("ERROR: Could not handle entry while walking components directory tree: {}", e)
                };

                // Remove read-only permissions on every entry
                let md = &entry.path().metadata().
                    expect("ERROR: Could not get metadata.");
                let mut perms = md.permissions();
                perms.set_readonly(false);
                fs::set_permissions(&entry.path(), perms)
                    .expect("Error: Failed to set permissions on .git directory");
            }

            fs::remove_dir_all(component_dir)
                .expect("ERROR: not able to delete component directory.");
        }
        else {
            // Use npm to remove the remote component
            npm_sr::npm_uninstall(name);
        }

        println!("{} component removed.", name);
    }

    /*
     * Adds a remote component via URL to node_modules.
     */
    pub fn add_remote_component(url: &str) {
        npm_sr::npm_install(&url);
    }

    /*
     * Downloads (copies) a component from a remote repository.
     */
    pub fn download_component(url: &str) {
        git_sr::git_clone(url);
    }

    /*
     * Updates all remote components in node_modules
     */
    pub fn update_dependencies() {
        npm_sr::npm_install("");
    }

    /*
     * Updates the local component who's directory we're in
     */
    pub fn update_local_component() {
        if Path::new(".git").exists() {
            git_sr::git_pull();
        }
    }


    /*
    * Generates a template README.md file to help the user get started.
    */
    fn generate_readme(name: &str) {
        if !Path::new("README.md").exists() {
            // Add the things that need to be put substituted into the README file
            let mut globals = liquid::value::Object::new();
            globals.insert("name".into(), liquid::value::Value::scalar(name.to_owned()));

            let contents = render_template("README.md.liquid", &mut globals);

            // Write the template text into the readme file
            match fs::write("README.md", contents) {
                Ok(res) => res,
                Err(error) => {
                    panic!("ERROR: Could not write to README.md file: {:?}", error);
                } 
            };
        }
        else {
            println!("README.md already exists, using existing file and refusing to overwrite.");
        }
    }


    /*
    * Generates a bill of materials from a template.
    */
    fn generate_bom(name: &str) {
        if !Path::new("bom_data.yaml").exists() {
            // Add the things that need to be put substituted into the BoM file
            let mut globals = liquid::value::Object::new();
            globals.insert("name".into(), liquid::value::Value::scalar(name.to_owned()));

            let contents = render_template("bom_data.yaml.liquid", &mut globals);

            // Write the template text into the readme file
            match fs::write("bom_data.yaml", contents) {
                Ok(res) => res,
                Err(error) => {
                    panic!("ERROR: Could not write to bom_data.yaml: {:?}", error);
                } 
            };
        }
        else {
            println!("bom_data.yaml already exists, using existing file and refusing to overwrite.");
        }
    }


    /*
     * Generates a package.json file for npm based on a Liquid template.
     */
    fn generate_package_json(name: &str, license: &str) {
        if !Path::new("package.json").exists() {
            // Add the things that need to be put substituted into the package file
            let mut globals = liquid::value::Object::new();
            globals.insert("name".into(), liquid::value::Value::scalar(name.to_owned()));
            globals.insert("license".into(), liquid::value::Value::scalar(license.to_owned()));

            let contents = render_template("package.json.liquid", &mut globals);

            // Write the contents into the file
            match fs::write("package.json", contents) {
                Ok(res) => res,
                Err(error) => {
                    panic!("ERROR: Could not write to package.json: {:?}", error);
                }
            };
        }
        else {
            println!("package.json already exists, using existing file and refusing to overwrite.");
        }
    }


    /*
    * Generates the .gitignore file used by the git command to ignore files and directories.
    */
    fn generate_gitignore() {
        if !Path::new(".gitignore").exists() {
            // Add the things that need to be put substituted into the gitignore file (none at this time)
            let mut globals = liquid::value::Object::new();

            let contents = render_template(".gitignore.liquid", &mut globals);

            // Write the contents to the file
            match fs::write(".gitignore", contents) {
                Ok(res) => res,
                Err(error) => {
                    panic!("ERROR: Could not write to .gitignore: {:?}", error);
                }
            };
        }
        else {
            println!(".gitignore already exists, using existing file and refusing to overwrite.");
        }
    }


    /*
    * Generates the dot file that tracks whether this is a top level component/project or a sub-component
    */
    fn generate_dot_file(is_top: bool, source_license: &str, doc_license: &str) {
        if !Path::new(".top").exists() {
            // Add the things that need to be put substituted into the .top file (none at this time)
            let mut globals = liquid::value::Object::new();
            globals.insert("is_top".into(), liquid::value::Value::scalar(is_top.to_owned()));
            globals.insert("source_license".into(), liquid::value::Value::scalar(source_license.to_owned()));
            globals.insert("doc_license".into(), liquid::value::Value::scalar(doc_license.to_owned()));

            let contents = render_template(".sr.liquid", &mut globals);

            // Write the contents to the file
            match fs::write(".sr", contents) {
                Ok(res) => res,
                Err(error) => {
                    panic!("ERROR: Could not write to .top: {:?}", error);
                }
            };
        }
        else {
            println!(".sr already exists, using existing file and refusing to overwrite.");
        }
    }


    /*
    * Reads a template to a string so that it can be written to a new components directory structure.
    */
    fn render_template(template_name: &str, globals: &mut liquid::value::Object) -> String {
        // Figure out where the templates are stored
        let template_file = match env::current_exe() {
            Ok(path) => path,
            Err(e) => panic!("ERROR: Could not get sliderule-cli executable directory: {}", e)
        };
        let template_file = match template_file.parent() {
            Some(path) => path,
            None => panic!("ERROR: Could not get parent of sliderule-cli executable directory.")
        };
        let template_file = template_file.join("templates").join(template_name);

        // Read the template file into a string so that it can be rendered using Liquid
        let mut file = fs::File::open(&template_file).expect("Unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the file");

        // Render the output of the template using Liquid
        let template = match liquid::ParserBuilder::with_liquid()
            .build()
            .parse(&contents) {
                Ok(temp) => temp,
                Err(e) => panic!("ERROR: Could not parse template using Liquid: {}", e)
            };

        let output = match template.render(globals) {
            Ok(out) => out,
            Err(e) => panic!("ERROR: Could not render template using Liquid: {}", e)
        };

        output
    }


    /*
    * Gets the current working directory for us, and handles any errors.
    */
    fn get_cwd() -> PathBuf {
        let path = env::current_dir();

        let cwd = match path {
            Ok(dir) => dir,
            Err(e) => {
                panic!("ERROR: Could not get current working directory: {}", e);
        }
        };

        cwd
    }


    /*
    * Attempts to extract the license from the package.json file in the current directory.
    */
    fn get_license(is_top: bool) -> String {
        let mut license = String::new();
        let sr_file = Path::new(".sr");

        println!("Attempting to extract license from package.json.");

        // Attempt to read the contents of package.json
        let mut file = fs::File::open(&sr_file).expect("ERROR: Unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("ERROR: Unable to read the file");

        // Make sure that there's a license entry in the package.json file for us to extract
        if !contents.contains("license") {
            panic!("ERROR: No package.json file to extract the license from.");
        }

        // Step through all the lines and attempt to find the license entry
        let lines = contents.split("\n");
        for line in lines {
            // Make sure that we're extracting the proper license at the proper time
            if line.contains("source_license") && !is_top {
                continue;
            }
            else if line.contains("documentation_license") && is_top {
                continue;
            }

            if line.contains("source_license") || line.contains("documentation_license") {
                let part: Vec<&str> = line.split(":").collect();
                license = String::from(part[1].replace("\"", "").trim());
                license = license.replace(",", "");
            }
        }

        license
    }
}

pub mod git_sr;
pub mod npm_sr;


#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
