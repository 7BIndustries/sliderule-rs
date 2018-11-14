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
        let source_license: String;
        let doc_license: String;

        // The path can either lead to a top level component (project), or a component nested within a project
        let component_dir: PathBuf;

        // This is a top level component (project)
        if Path::new(".sr").exists() {
            component_dir = PathBuf::from("components").join(name);
        }
        else {
            component_dir = PathBuf::from(name);
        }

        // Create a directory for our component
        fs::create_dir(&component_dir)
            .expect("Could not create dist directory.");

        // Make a new directory in components, cd into it, and then run the rest of this code
        env::set_current_dir(&component_dir)
            .expect("Could not change into components directory.");

        // Create the components directory, if needed
        if !Path::new("components").exists() {
            fs::create_dir("components")
                .expect("Could not create components directory.");
        }
        else {
            println!("components directory already exists, using existing directory.");
        }

        // Create the dist directory, if needed
        if !Path::new("dist").exists() {
            fs::create_dir("dist")
                .expect("Could not create dist directory.");
        }
        else {
            println!("dist directory already exists, using existing directory.");
        }

        // Create the docs directory, if needed
        if !Path::new("docs").exists() {
            fs::create_dir("docs")
                .expect("Could not create docs directory.");
        }
        else {
            println!("docs directory already exists, using existing directory.");
        }

        //Create the source directory, if needed
        if !Path::new("source").exists() {
            fs::create_dir("source")
                .expect("Could not create source directory.");
        }
        else {
            println!("source directory already exists, using existing directory.");
        }

        // Generate the template readme file
        generate_readme(&name);

        // Generate bom_data.yaml
        generate_bom(&name);

        // Ask the user for their license choice for the source of this component if they haven't specified it on the command line
        let licenses = get_licenses();

        // See if the source license was supplied in a command line argument
        if src_license.is_empty() {
            source_license = licenses.0;
        }
        else {
            source_license = src_license.to_string();
        }

        // See if the documentation license was supplied in a command line argument
        if docs_license.is_empty() {
            doc_license = licenses.1;
        }
        else {
            doc_license = docs_license.to_string();
        }

        // Generate package.json, if needed
        generate_package_json(&name, &source_license);

        // Generate the .sr file that provides extra information about this component
        generate_dot_file(&source_license, &doc_license);

        // Make sure that our package.json file is updated with all the license info
        amalgamate_licenses();

        println!("Finished setting up component.");
    }


    /*
     * Uploads any changes to the project to the remote repository.
     */
    pub fn upload_project() {
        let mut url = String::new();

        // Make sure that our package.json file is updated with all the license info
        amalgamate_licenses();

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
            env::set_current_dir(&component_dir)
                .expect("Could not change into components directory.");

            // Set the directory up as a git repo and then push the changes to the remote
            git_sr::git_init(&url.trim());
            git_sr::git_add_and_commit();

            // Change back up to the original, top level directory
            env::set_current_dir(&orig_dir)
                .expect("Could not change into original parent directory.");

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
        let component_dir = Path::new("components").join(name);

        // If the component exists as a subdirectory of components delete the directory directly otherwise use npm to remove it.
        if component_dir.exists() {
            println!("Deleting component directory.");

            // Step through every file and directory in the path to be deleted and make sure that none are read-only
            for entry in walkdir::WalkDir::new(&component_dir) {
                let entry = entry
                    .expect("Could not handle entry while walking components directory tree.");

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
     * Allows the user to change the source and/or documentation licenses for the project.
     */
    pub fn change_licenses(source_license: &String, doc_license: &String) {
        let cwd = get_cwd().join(".sr");

        update_yaml_value(&cwd, "source_license", source_license);
        update_yaml_value(&cwd, "documentation_license", doc_license);

        // Make sure our new licenses are up to date in package.json
        amalgamate_licenses();
    }

    /*
     * Adds a remote component via URL to node_modules.
     */
    pub fn add_remote_component(url: &str) {
        npm_sr::npm_install(&url);

        // Make sure that our package.json file is updated with all the license info
        amalgamate_licenses();
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
            fs::write("README.md", contents)
                .expect("Could not write to README.md file.");
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
            fs::write("bom_data.yaml", contents)
                .expect("Could not write to bom_data.yaml.");
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
            fs::write("package.json", contents)
                .expect("Could not write to package.json.");
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
            fs::write(".gitignore", contents)
                .expect("Could not write to .gitignore.");
        }
        else {
            println!(".gitignore already exists, using existing file and refusing to overwrite.");
        }
    }


    /*
    * Generates the dot file that tracks whether this is a top level component/project or a sub-component
    */
    fn generate_dot_file(source_license: &str, doc_license: &str) {
        if !Path::new(".sr").exists() {
            // Add the things that need to be put substituted into the .top file (none at this time)
            let mut globals = liquid::value::Object::new();
            globals.insert("source_license".into(), liquid::value::Value::scalar(source_license.to_owned()));
            globals.insert("doc_license".into(), liquid::value::Value::scalar(doc_license.to_owned()));

            let contents = render_template(".sr.liquid", &mut globals);

            // Write the contents to the file
            fs::write(".sr", contents)
                .expect("Could not write to .sr file.");
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
        let template_file = env::current_exe()
            .expect("Could not get sliderule-cli executable directory.");

        let template_file = template_file.parent()
            .expect("Could not get parent of sliderule-cli executable directory.");

        let template_file = template_file.join("templates").join(template_name);

        // Read the template file into a string so that it can be rendered using Liquid
        let mut file = fs::File::open(&template_file).expect("Unable to open the file");
        let mut contents = String::new();
        file.read_to_string(&mut contents).expect("Unable to read the file");

        // Render the output of the template using Liquid
        let template = liquid::ParserBuilder::with_liquid()
            .build()
            .parse(&contents)
                .expect("Could not parse template using Liquid.");

        let output = template.render(globals)
            .expect("Could not render template using Liquid.");

        output
    }

    /*
     * Extracts the source and documentation licenses from a component's .sr file.
     */
    pub fn get_licenses() -> (String, String) {
        let sr_file: PathBuf;

        // We can hand back the default licenses, if nothing else
        let mut source_license = String::from("Unlicense");
        let mut doc_license = String::from("CC-BY-4.0");

        // If we're in a component directory, pull the license info from that
        sr_file = get_cwd().join(".sr");

        // Safety check to make sure the file exists
        if sr_file.exists() {
            println!("Attempting to extract license from {}", sr_file.display());

            // Extract the licenses from the file
            source_license = get_yaml_value(&sr_file, "source_license");
            doc_license = get_yaml_value(&sr_file, "documentation_license");
        }

        (source_license, doc_license)
    }

    /*
     * Walk the directory structure of the current component and combine the licenses per the SPDX naming conventions.
     */
    fn amalgamate_licenses() {
        let mut license_str = String::new();
        let mut source_licenses: Vec<String> = Vec::new();
        let mut doc_licenses: Vec<String> = Vec::new();

        let cur_dir = get_cwd();

        // Walk through every sub-directory in this component, looking for .sr files
        for entry in walkdir::WalkDir::new(&cur_dir) {
            let entry = entry
                .expect("Could not handle entry while walking components directory tree.");

            // If we have a .sr file, keep it for later license extraction
            if entry.path().ends_with(".sr") {
                // We want the licenses from our current dot files
                let source_value = get_yaml_value(&entry.path().to_path_buf(), "source_license");
                let doc_value = get_yaml_value(&entry.path().to_path_buf(), "documentation_license");

                // Keep track of the license strings, avoiding duplicates
                if !source_licenses.contains(&source_value) {
                    source_licenses.push(source_value);
                }
                if !doc_licenses.contains(&doc_value) {
                    doc_licenses.push(doc_value);
                }
            }
        }

        // We talked the directories in a way that the entries will be in reverse order
        source_licenses.reverse();
        doc_licenses.reverse();

        // Step through all of the source licenses and append them to the license string
        let mut i = 0;
        for lic in source_licenses {
            // Make sure that the list is AND-concatenated
            if i > 0 {
                license_str.push_str(" AND ")
            }

            license_str.push_str(&lic);

            i = i + 1;
        }

        // Make sure that there's an AND concatenation after the source license
        if doc_licenses.len() > 0 && i > 0 {
            license_str.push_str(" AND ");
        }

        // Step through all of the documentation licenses and append them to the license string
        let mut j = 0;
        for lic in doc_licenses {
            // Make sure that the list is AND-concatenated
            if j > 0 {
                license_str.push_str(" AND ");
            }

            license_str.push_str(&lic);

            j = j + 1;
        }

        update_json_value(&cur_dir.join("package.json"), "license", &license_str);
    }

    /*
     * Extracts a value from a JSON file based on a string key.
     */
    // fn get_json_value(json_file: &PathBuf, key: &str) -> String {
    //     let mut value = String::new();

    //     // If the file doesn't exist, we can't do anything
    //     if json_file.exists() {
    //         // Open the file for reading
    //         let mut file = fs::File::open(&json_file)
    //             .expect("Error opening JSON file.");

    //         // Attempt to read the contents of the file
    //         let mut contents = String::new();
    //         file.read_to_string(&mut contents).expect("ERROR: Unable to read the JSON file for this component");

    //         let lines = contents.split("\n");
    //         for line in lines {
    //             // Make sure that we're extracting the proper license at the proper time
    //             if line.contains(&key) {
    //                 let part: Vec<&str> = line.split(":").collect();
    //                 value = part[1].replace("\"", "").replace(",", "").trim().to_string();
    //             }
    //         }
    //     }
    //     else {
    //         panic!("JSON file {} not found, cannot extract data from it.", json_file.display());
    //     }

    //     value
    // }

    /*
     * Replaces the value corresponding to a key in a JSON file
     */
    fn update_json_value(json_file: &PathBuf, key: &str, value: &str) {
        if json_file.exists() {
            println!("Attempting to update value for {} in {}.", json_file.display(), key);

            // Open the file for reading
            let mut file = fs::File::open(&json_file)
                .expect("Error opening JSON file.");

            // Attempt to read the contents of the component's .sr file
            let mut contents = String::new();
            let mut new_contents = String::new();
            file.read_to_string(&mut contents).expect("ERROR: Unable to read the JSON file for this component");

            let lines = contents.split("\n");
            for line in lines {
                // Make sure that we're extracting the proper license at the proper time
                if line.contains(&key) {
                    // Grab the original value
                    let part: Vec<&str> = line.split(":").collect();
                    let old_value = part[1].replace("\"", "").replace(",", "").trim().to_string();

                    // Scope the change to matching line and replace the original line with the new one
                    let new_line = line.replace(&old_value, &value);
                    new_contents = contents.replace(line, &new_line);
                }
            }

            // Make sure there's a change to write
            if !new_contents.is_empty() {
                // Try to write the contents back to the file
                fs::write(json_file, new_contents)
                    .expect("Could not write to JSON file.");
            }
        }
    }

    /*
     * Extracts a value from a yaml file based on a string key.
     */
    fn get_yaml_value(yaml_file: &PathBuf, key: &str) -> String {
        let mut value = String::new();

        // If the file doesn't exist, we can't do anything
        if yaml_file.exists() {
            // Open the file for reading
            let mut file = fs::File::open(&yaml_file)
                .expect("Error opening yaml file.");

            // Attempt to read the contents of the file
            let mut contents = String::new();
            file.read_to_string(&mut contents).expect("ERROR: Unable to read the yaml file for this component");

            let lines = contents.split("\n");
            for line in lines {
                // Make sure that we're extracting the proper license at the proper time
                if line.contains(&key) {
                    let part: Vec<&str> = line.split(":").collect();
                    value = String::from(part[1].replace(",", "").trim());
                }
            }
        }
        else {
            panic!("yaml file {} not found, cannot extract data from it.", yaml_file.display());
        }

        value
    }

    /*
     * Replaces the value corresponding to a key in a yaml file
     */
    fn update_yaml_value(yaml_file: &PathBuf, key: &str, value: &str) {
        if yaml_file.exists() {
            // Open the file for reading
            let mut file = fs::File::open(&yaml_file)
                .expect("Error opening yaml file.");

            // Attempt to read the contents of the component's .sr file
            let mut contents = String::new();
            let mut new_contents = String::new();
            file.read_to_string(&mut contents).expect("ERROR: Unable to read the yaml file for this component");

            let lines = contents.split("\n");
            for line in lines {
                // Make sure that we're extracting the proper license at the proper time
                if line.contains(&key) {
                    // Grab the original value
                    let part: Vec<&str> = line.split(":").collect();
                    let old_value = String::from(part[1].replace(",", "").trim());

                    // Scope the change to matching line and replace the original line with the new one
                    let new_line = line.replace(&old_value, &value);
                    new_contents = contents.replace(line, &new_line);
                }
            }

            // Make sure there's a change to write
            if !new_contents.is_empty() {
                // Try to write the contents back to the file
                fs::write(yaml_file, new_contents)
                    .expect("Could not write to yaml file.");
            }
        }
    }

    /*
    * Gets the current working directory for us, and handles any errors.
    */
    fn get_cwd() -> PathBuf {
        let path = env::current_dir();

        let cwd = path
            .expect("Could not get current working directory.");

        cwd
    }

    // Gets the parent directory of the current component
    fn get_parent_dir() -> PathBuf {
        // Get the current directory
        let cur_dir = get_cwd();

        // Get the parent directory of this component's directory
        let parent_dir = cur_dir.parent()
            .expect("Could not get the parent directory of the current component.");

        parent_dir.to_path_buf()
    }

    /*
     * Figures out what depth the component is at.
     * 0 = A top level component is probably being created
     * 1 = A top level component with no parent
     * 2 = A sub-component at depth n
     */
    pub fn get_level() -> u8 {
        let level: u8;

        // Allows us to check if there is a .sr file in the current directory
        let current_file = get_cwd().join(".sr");
    
        // Allows us to check if there is a .sr file in the parent directory
        let parent_file = get_parent_dir().join(".sr");

        // If the parent directory contains a .sr file, we have a sub-component, if not we have a top level component
        if !parent_file.exists() && !current_file.exists() {
            level = 0;
        }
        else if !parent_file.exists() && current_file.exists() {
            level = 1;
        }
        else {
            level = 2;
        }

        level
    }
}

pub mod git_sr;
pub mod npm_sr;


#[cfg(test)]
mod tests {
    use sliderule;

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
