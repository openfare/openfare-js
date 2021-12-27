use super::common;
use anyhow::{format_err, Result};
use openfare_lib::extension::Extension;

pub fn package_dependencies_configs(
    extension: &crate::JsExtension,
    package_name: &str,
    package_version: &Option<&str>,
    _extension_args: &Vec<String>,
) -> Result<
    openfare_lib::extension::commands::package_dependencies_configs::PackageDependenciesConfigs,
> {
    let package_version = match package_version {
        Some(v) => Some(v.to_string()),
        None => common::get_latest_version(&package_name)?,
    };

    let tmp_dir = tempdir::TempDir::new("openfare_js")?;
    let tmp_dir = tmp_dir.path().to_path_buf();

    install_package(&package_name, &package_version, &tmp_dir)?;

    let primary_package_directory = tmp_dir.join("node_modules").join(package_name);

    let package = common::npm::get_package(&primary_package_directory.join("package.json"))?;
    let config = common::npm::get_config(&primary_package_directory)?;

    let node_modules_directory = tmp_dir.join("node_modules");
    let mut dependencies_configs = common::npm::get_node_modules_configs(&node_modules_directory)?;
    dependencies_configs.remove(&package);

    Ok(openfare_lib::extension::commands::package_dependencies_configs::PackageDependenciesConfigs {
        registry_host_name: extension.registries().first().ok_or(format_err!("Code error: at least one registry host name expected."))?.to_string(),
        package_configs: openfare_lib::package::PackageConfigs {
            primary_package: Some(package),
            primary_package_config: config,
            dependencies_configs: dependencies_configs,
        },
    })
}

/// Execute npm install in tmp directory.
///
/// Example command: npm install is-even@1.0.0
fn install_package(
    package_name: &str,
    package_version: &Option<String>,
    tmp_dir: &std::path::PathBuf,
) -> Result<()> {
    let package = if let Some(package_version) = package_version {
        format!(
            "{name}@{version}",
            name = package_name,
            version = package_version
        )
    } else {
        package_name.to_string()
    };

    log::debug!(
        "Executing npm install in temp directory (exists: {}): {}",
        tmp_dir.exists(),
        tmp_dir.display()
    );
    std::process::Command::new("npm")
        .args(vec!["install", package.as_str()])
        .stdin(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .current_dir(&tmp_dir)
        .output()?;
    log::debug!("Finished executing npm install.");
    Ok(())
}
