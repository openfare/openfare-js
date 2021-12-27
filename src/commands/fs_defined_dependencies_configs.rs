use super::common;
use anyhow::{format_err, Result};
use openfare_lib::extension::commands::fs_defined_dependencies_configs::FsDefinedDependenciesConfigs;

pub fn fs_defined_dependencies_configs(
    working_directory: &std::path::PathBuf,
    _extension_args: &Vec<String>,
) -> Result<FsDefinedDependenciesConfigs> {
    // Identify all dependency definition files.
    let dependency_files = match common::identify_dependency_files(&working_directory) {
        Some(v) => v,
        None => {
            log::debug!("Did not identify any dependency definition files.");
            return Ok(FsDefinedDependenciesConfigs::default());
        }
    };
    let dependency_file = match dependency_files.first() {
        Some(f) => f,
        None => {
            log::debug!("Did not identify any dependency definition files.");
            return Ok(FsDefinedDependenciesConfigs::default());
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

    let primary_package = common::npm::get_package(&dependency_file.path)?;
    let primary_package_config = common::npm::get_config(&project_path)?;

    let node_modules_directories = common::identify_node_modules_directories(&working_directory)?;

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

        common::identify_node_modules_directories(&working_directory)?
    };

    let node_modules_directory = node_modules_directories.local;

    let dependencies_configs = if let Some(node_modules_directory) = node_modules_directory {
        common::npm::get_node_modules_configs(&node_modules_directory)?
    } else {
        log::debug!("Failed to find `node_modules` directory.");
        openfare_lib::package::DependenciesConfigs::new()
    };

    Ok(FsDefinedDependenciesConfigs {
        project_path: project_path.to_path_buf(),
        package_configs: openfare_lib::package::PackageConfigs {
            primary_package: Some(primary_package),
            primary_package_config: primary_package_config,
            dependencies_configs,
        },
    })
}
