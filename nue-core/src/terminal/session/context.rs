use std::collections::BTreeMap;
use std::path::{Component, Path, PathBuf};

use crate::terminal::session::{RunCommandContextError, RunCommandRequest};

pub(super) fn normalize_path(path: &Path) -> Option<PathBuf> {
    let mut normalized = PathBuf::new();
    let mut normal_segments = 0;

    for component in path.components() {
        match component {
            Component::Prefix(prefix) => normalized.push(prefix.as_os_str()),
            Component::RootDir => normalized.push(component.as_os_str()),
            Component::CurDir => {}
            Component::ParentDir => {
                if normal_segments == 0 {
                    return None;
                }
                normalized.pop();
                normal_segments -= 1;
            }
            Component::Normal(part) => {
                normalized.push(part);
                normal_segments += 1;
            }
        }
    }

    Some(normalized)
}

pub(super) fn validate_run_command_context(
    workspace_root: &Path,
    workspace_env: &BTreeMap<String, String>,
    request: &RunCommandRequest,
) -> Result<(), RunCommandContextError> {
    if let Some(cwd) = &request.cwd {
        if cwd
            .components()
            .any(|component| matches!(component, Component::ParentDir))
        {
            return Err(RunCommandContextError::InvalidCwd {
                attempted: cwd.clone(),
                workspace_root: workspace_root.to_path_buf(),
            });
        }

        let normalized = normalize_path(cwd).ok_or_else(|| RunCommandContextError::InvalidCwd {
            attempted: cwd.clone(),
            workspace_root: workspace_root.to_path_buf(),
        })?;

        if normalized != workspace_root {
            return Err(RunCommandContextError::InvalidCwd {
                attempted: cwd.clone(),
                workspace_root: workspace_root.to_path_buf(),
            });
        }
    }

    for (key, value) in request.env_overrides.iter() {
        match workspace_env.get(key) {
            None => {
                return Err(RunCommandContextError::UnauthorizedEnvAddition { key: key.clone() });
            }
            Some(expected) if expected != value => {
                return Err(RunCommandContextError::UnauthorizedEnvModification {
                    key: key.clone(),
                    expected: expected.clone(),
                    attempted: value.clone(),
                });
            }
            _ => {}
        }
    }

    Ok(())
}
