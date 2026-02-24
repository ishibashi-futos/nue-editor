use std::error::Error;
use std::fmt;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppLaunchConfigOverrides {
    pub workspace_root: Option<PathBuf>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AppLaunchConfig {
    workspace_root: Option<PathBuf>,
}

impl AppLaunchConfig {
    pub fn workspace_root(&self) -> Option<&Path> {
        self.workspace_root.as_deref()
    }
}

pub fn resolve_app_launch_config(
    overrides: AppLaunchConfigOverrides,
) -> Result<AppLaunchConfig, AppLaunchConfigResolveError> {
    if let Some(path) = &overrides.workspace_root
        && !path.is_absolute()
    {
        return Err(AppLaunchConfigResolveError::WorkspacePathMustBeAbsolute {
            path: path.clone(),
        });
    }

    Ok(AppLaunchConfig {
        workspace_root: overrides.workspace_root,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AppLaunchConfigResolveError {
    WorkspacePathMustBeAbsolute { path: PathBuf },
}

impl fmt::Display for AppLaunchConfigResolveError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::WorkspacePathMustBeAbsolute { path } => write!(
                f,
                "`--workspace` には絶対パスを指定してください: {}",
                path.display()
            ),
        }
    }
}

impl Error for AppLaunchConfigResolveError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_config_has_no_workspace_root() {
        let config = resolve_app_launch_config(AppLaunchConfigOverrides::default())
            .expect("default config should resolve");

        assert_eq!(config.workspace_root(), None);
    }

    #[test]
    fn absolute_workspace_root_is_accepted() {
        let config = resolve_app_launch_config(AppLaunchConfigOverrides {
            workspace_root: Some(PathBuf::from("/tmp/workspace")),
        })
        .expect("absolute path should resolve");

        assert_eq!(config.workspace_root(), Some(Path::new("/tmp/workspace")));
    }

    #[test]
    fn relative_workspace_root_is_rejected() {
        let error = resolve_app_launch_config(AppLaunchConfigOverrides {
            workspace_root: Some(PathBuf::from("relative/workspace")),
        })
        .expect_err("relative path should fail");

        assert!(matches!(
            error,
            AppLaunchConfigResolveError::WorkspacePathMustBeAbsolute { .. }
        ));
        assert!(error.to_string().contains("絶対パス"));
    }
}
