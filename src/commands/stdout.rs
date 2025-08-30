use crate::commands::change_directory::change_directory;
use crate::error::{AppError, AppErrorKind, AppResult};
use crate::types::state::AppState;
use crate::utils::unix::expand_shell_string;
use std::collections::HashSet;
use std::path::PathBuf;

use std::process::Command;

use super::select::select_by_names;

#[derive(Debug, Clone)]
pub enum PostProcessor {
    ChangeDirectory,
    SelectNames,
}

impl PostProcessor {
    pub fn from_str(args: &str) -> Option<Self> {
        match args {
            "cd" => Some(PostProcessor::ChangeDirectory),
            "select-names" => Some(PostProcessor::SelectNames),
            _ => None,
        }
    }
}

fn assert_one_line(stdout: &str) -> AppResult {
    match stdout.lines().count() {
        1 => Ok(()),
        _ => Err(AppError::new(AppErrorKind::StateError, "The last `capture` stdout does not have exactly one line as expected for this stdout processor".to_string()))
    }
}

fn as_one_existing_directory(stdout: &str) -> AppResult<PathBuf> {
    assert_one_line(stdout)?;
    let path = expand_shell_string(stdout);
    if path.exists() {
        if path.is_file() {
            if let Some(parent) = path.parent() {
                Ok(parent.to_path_buf())
            } else {
                Err(AppError::new(AppErrorKind::StateError, "The last `capture` output is a file but without a valid directory as parent in the file system".to_string()))
            }
        } else {
            Ok(path.to_path_buf())
        }
    } else {
        Err(AppError::new(
            AppErrorKind::StateError,
            "The last `capture` output line is not an existing path".to_string(),
        ))
    }
}

fn as_string_hash(stdout: &str) -> AppResult<HashSet<String>> {
    Ok(stdout.lines().map(|s| s.to_string()).collect())
}

pub fn post_process_std_out(processor: &PostProcessor, app_state: &mut AppState) -> AppResult {
    let last_stdout = &app_state.state.last_stdout;
    if let Some(stdout) = last_stdout {
        let stdout = stdout.trim();
        match processor {
            PostProcessor::ChangeDirectory => {
                change_directory(app_state, as_one_existing_directory(stdout)?.as_path())
            },
            PostProcessor::SelectNames => {
                select_by_names(app_state, &as_string_hash(stdout)?, true)
            },
        }
    } else {
        Err(AppError::new(
            AppErrorKind::StateError,
            "No result from a former `shell` available".to_string(),
        ))
    }
}
