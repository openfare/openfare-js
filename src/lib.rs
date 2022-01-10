use anyhow::Result;

mod commands;

#[derive(Clone, Debug)]
pub struct JsExtension {
    name_: String,
    registry_host_names_: Vec<String>,
}

impl openfare_lib::extension::FromLib for JsExtension {
    fn new() -> Self {
        Self {
            name_: "js".to_string(),
            registry_host_names_: vec!["npmjs.com".to_owned()],
        }
    }
}

impl openfare_lib::extension::Extension for JsExtension {
    fn name(&self) -> String {
        self.name_.clone()
    }

    fn registries(&self) -> Vec<String> {
        self.registry_host_names_.clone()
    }

    fn package_dependencies_locks(
        &self,
        package_name: &str,
        package_version: &Option<&str>,
        extension_args: &Vec<String>,
    ) -> Result<
        openfare_lib::extension::commands::package_dependencies_locks::PackageDependenciesLocks,
    > {
        commands::package_dependencies_locks(
            &self,
            &package_name,
            &package_version,
            &extension_args,
        )
    }

    fn fs_defined_dependencies_locks(
        &self,
        working_directory: &std::path::PathBuf,
        extension_args: &Vec<String>,
    ) -> Result<openfare_lib::extension::commands::fs_defined_dependencies_locks::FsDefinedDependenciesLocks>{
        commands::fs_defined_dependencies_locks(&working_directory, &extension_args)
    }
}
