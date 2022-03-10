use anyhow::{format_err, Result};
use openfare_lib::extension::commands::project_dependencies_locks::ProjectDependenciesLocks;

pub fn project_dependencies_locks(
    working_directory: &std::path::PathBuf,
    _extension_args: &Vec<String>,
) -> Result<ProjectDependenciesLocks> {
    // Identify all dependency definition files.
    let dependency_files =
        match crate::registries::npm::identify_dependency_files(&working_directory) {
            Some(v) => v,
            None => {
                log::debug!("Did not identify any dependency definition files.");
                return Ok(ProjectDependenciesLocks::default());
            }
        };
    let dependency_file = match dependency_files.first() {
        Some(f) => f,
        None => {
            log::debug!("Did not identify any dependency definition files.");
            return Ok(ProjectDependenciesLocks::default());
        }
    };

    log::debug!(
        "Found dependency definitions file: {}",
        dependency_file.path.display()
    );

    let project_path = dependency_file
        .path
        .parent()
        .ok_or(format_err!(
            "Failed to derive parent directory from dependency file path: {}",
            dependency_file.path.display()
        ))?
        .to_path_buf();

    let primary_package = crate::registries::npm::get_package(&dependency_file.path)?;
    let primary_package_lock = crate::registries::npm::get_lock(&project_path)?;

    let node_modules_directories =
        crate::registries::npm::identify_node_modules_directories(&working_directory)?;

    // if failed to find node modules directory, search again after running npm install.
    let node_modules_directories = if node_modules_directories.local.is_some() {
        log::debug!("Found `node_modules` directory.");
        node_modules_directories
    } else {
        log::debug!(
            "Failed to find `node_modules` directory. Attempting to generate using `npm install`."
        );
        std::process::Command::new("npm")
            .args(vec!["install", "--prod"])
            .stdin(std::process::Stdio::null())
            .stderr(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .current_dir(&project_path)
            .output()?;

        crate::registries::npm::identify_node_modules_directories(&working_directory)?
    };

    let node_modules_directory = node_modules_directories.local;

    let dependencies_locks = if let Some(node_modules_directory) = node_modules_directory {
        crate::registries::npm::get_node_modules_locks(&node_modules_directory)?
    } else {
        log::debug!("Failed to find `node_modules` directory.");
        openfare_lib::package::DependenciesLocks::new()
    };

    Ok(ProjectDependenciesLocks {
        project_path: project_path.to_path_buf(),
        package_locks: openfare_lib::package::PackageLocks {
            primary_package: Some(primary_package),
            primary_package_lock,
            dependencies_locks,
        },
    })
}
