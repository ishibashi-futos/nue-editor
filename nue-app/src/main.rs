use std::env;
use std::ffi::OsString;
use std::fmt;
use std::path::PathBuf;
use std::process;

use anyhow::{Context, Result};
use nue_config::{AppLaunchConfig, AppLaunchConfigCliInput};
use tokio::runtime::{Builder, Runtime};

type LaunchConfig = AppLaunchConfig;

fn main() {
    match try_main(env::args_os()) {
        Ok(()) => {}
        Err(error) => {
            eprintln!("{error}");
            process::exit(error.exit_code());
        }
    }
}

fn try_main(args: impl IntoIterator<Item = OsString>) -> Result<(), AppError> {
    let command = prepare_command(args)?;
    match command {
        AppCommand::Help { program_name } => {
            println!("{}", help_text(&program_name));
            Ok(())
        }
        AppCommand::Run(config) => {
            let runtime = build_tokio_runtime().map_err(AppError::runtime)?;
            runtime
                .block_on(launch_ui(config))
                .map_err(AppError::ui_launch)
        }
    }
}

fn prepare_command(args: impl IntoIterator<Item = OsString>) -> Result<AppCommand, AppError> {
    let parsed = parse_cli_args(args)?;
    match parsed {
        ParsedCli::Help { program_name } => Ok(AppCommand::Help { program_name }),
        ParsedCli::Args {
            program_name,
            cli_args,
        } => resolve_launch_config(cli_args)
            .map(AppCommand::Run)
            .map_err(|error| AppError::usage(program_name, error)),
    }
}

fn build_tokio_runtime() -> Result<Runtime> {
    Builder::new_multi_thread()
        .enable_all()
        .thread_name("nue-app-tokio")
        .build()
        .context("tokio ランタイムの初期化に失敗しました")
}

async fn launch_ui(config: LaunchConfig) -> Result<()> {
    let request =
        nue_ui::UiLaunchRequest::new().with_workspace_root(config.workspace_root().map(PathBuf::from));
    nue_ui::run_app(request)
}

fn resolve_launch_config(cli_args: CliArgs) -> Result<LaunchConfig, String> {
    let cli_input = AppLaunchConfigCliInput::new().with_workspace_root(cli_args.workspace_root);
    LaunchConfig::resolve(cli_input).map_err(|error| error.to_string())
}

fn parse_cli_args(args: impl IntoIterator<Item = OsString>) -> Result<ParsedCli, AppError> {
    let mut args = args.into_iter();
    let program_name = args
        .next()
        .map(|value| value.to_string_lossy().into_owned())
        .unwrap_or_else(|| "nue-app".to_string());

    let mut workspace_root: Option<PathBuf> = None;

    while let Some(raw_arg) = args.next() {
        if raw_arg == "-h" || raw_arg == "--help" {
            return Ok(ParsedCli::Help { program_name });
        }

        let arg = raw_arg.to_string_lossy();

        if let Some(value) = arg.strip_prefix("--workspace=") {
            let path = parse_workspace_value(value, &program_name)?;
            set_workspace_root(&mut workspace_root, path, &program_name)?;
            continue;
        }

        if arg == "--workspace" {
            let value = args.next().ok_or_else(|| {
                AppError::usage(
                    program_name.clone(),
                    "`--workspace` の値が指定されていません".to_string(),
                )
            })?;
            let path = parse_workspace_value_lossy(&value, &program_name)?;
            set_workspace_root(&mut workspace_root, path, &program_name)?;
            continue;
        }

        return Err(AppError::usage(
            program_name,
            format!("未対応の引数です: {arg}"),
        ));
    }

    Ok(ParsedCli::Args {
        program_name,
        cli_args: CliArgs { workspace_root },
    })
}

fn set_workspace_root(
    slot: &mut Option<PathBuf>,
    path: PathBuf,
    program_name: &str,
) -> Result<(), AppError> {
    if slot.is_some() {
        return Err(AppError::usage(
            program_name.to_string(),
            "`--workspace` は 1 回だけ指定できます".to_string(),
        ));
    }
    *slot = Some(path);
    Ok(())
}

fn parse_workspace_value(value: &str, program_name: &str) -> Result<PathBuf, AppError> {
    if value.is_empty() {
        return Err(AppError::usage(
            program_name.to_string(),
            "`--workspace` の値が空です".to_string(),
        ));
    }
    Ok(PathBuf::from(value))
}

fn parse_workspace_value_lossy(value: &OsString, program_name: &str) -> Result<PathBuf, AppError> {
    let text = value.to_string_lossy();
    parse_workspace_value(text.as_ref(), program_name)
}

fn help_text(program_name: &str) -> String {
    format!(
        "\
{program_name} - Nue App Host (起動骨格)

USAGE:
  {program_name} [OPTIONS]

OPTIONS:
  -h, --help                 このヘルプを表示する
      --workspace <PATH>     起動対象ワークスペースの絶対パス
"
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CliArgs {
    workspace_root: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ParsedCli {
    Help {
        program_name: String,
    },
    Args {
        program_name: String,
        cli_args: CliArgs,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum AppCommand {
    Help { program_name: String },
    Run(LaunchConfig),
}

#[derive(Debug)]
enum AppErrorKind {
    Usage,
    Runtime,
    UiLaunch,
}

#[derive(Debug)]
struct AppError {
    kind: AppErrorKind,
    message: String,
    program_name: Option<String>,
}

impl AppError {
    fn usage(program_name: String, message: String) -> Self {
        Self {
            kind: AppErrorKind::Usage,
            message,
            program_name: Some(program_name),
        }
    }

    fn runtime(error: anyhow::Error) -> Self {
        Self {
            kind: AppErrorKind::Runtime,
            message: error.to_string(),
            program_name: None,
        }
    }

    fn ui_launch(error: anyhow::Error) -> Self {
        Self {
            kind: AppErrorKind::UiLaunch,
            message: error.to_string(),
            program_name: None,
        }
    }

    fn exit_code(&self) -> i32 {
        match self.kind {
            AppErrorKind::Usage => 2,
            AppErrorKind::Runtime | AppErrorKind::UiLaunch => 1,
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.kind {
            AppErrorKind::Usage => {
                let program_name = self.program_name.as_deref().unwrap_or("nue-app");
                write!(
                    f,
                    "引数エラー: {}\n\n{}",
                    self.message,
                    help_text(program_name)
                )
            }
            AppErrorKind::Runtime => write!(f, "起動エラー: {}", self.message),
            AppErrorKind::UiLaunch => write!(f, "UI 起動エラー: {}", self.message),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_flag_returns_help_command() {
        let result = parse_cli_args(["nue-app".into(), "--help".into()]).expect("help parse");
        assert_eq!(
            result,
            ParsedCli::Help {
                program_name: "nue-app".to_string()
            }
        );
    }

    #[test]
    fn workspace_option_accepts_equals_syntax() {
        let result = prepare_command(["nue-app".into(), "--workspace=/tmp".into()])
            .expect("parse workspace");

        assert_eq!(
            result,
            AppCommand::Run(
                LaunchConfig::resolve(
                    AppLaunchConfigCliInput::new().with_workspace_root(Some(PathBuf::from("/tmp"))),
                )
                .expect("valid config"),
            )
        );
    }

    #[test]
    fn workspace_option_rejects_relative_path() {
        let error = prepare_command(["nue-app".into(), "--workspace".into(), "relative".into()])
            .expect_err("relative path should fail");

        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("絶対パス"));
    }

    #[test]
    fn unknown_argument_returns_usage_error() {
        let error = parse_cli_args(["nue-app".into(), "--unknown".into()])
            .expect_err("unknown flag should fail");

        assert_eq!(error.exit_code(), 2);
        assert!(error.to_string().contains("未対応の引数"));
    }
}
