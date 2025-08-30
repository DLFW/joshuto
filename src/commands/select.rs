use std::collections::HashSet;

use crate::error::AppResult;
use crate::types::state::{AppState, MatchState};

use std::process::Command;

use super::cursor_move;

#[derive(Clone, Copy, Debug)]
pub struct SelectOption {
    pub toggle: bool,
    pub all: bool,
    pub reverse: bool,
}

impl std::default::Default for SelectOption {
    fn default() -> Self {
        Self {
            toggle: true,
            all: false,
            reverse: false,
        }
    }
}

impl std::fmt::Display for SelectOption {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "--toggle={} --all={} --deselect={}",
            self.toggle, self.all, self.reverse
        )
    }
}

pub fn select_files(
    app_state: &mut AppState,
    pattern: &MatchState,
    options: &SelectOption,
) -> AppResult {
    if pattern.is_none() {
        select_without_pattern(app_state, options)
    } else {
        select_with_pattern(app_state, pattern, options)
    }
}

fn select_without_pattern(app_state: &mut AppState, options: &SelectOption) -> AppResult {
    if options.all {
        if let Some(curr_list) = app_state
            .state
            .tab_state_mut()
            .curr_tab_mut()
            .curr_list_mut()
        {
            curr_list.iter_mut().for_each(|e| {
                if options.reverse {
                    e.set_permanent_selected(false);
                } else if options.toggle {
                    e.set_permanent_selected(!e.is_selected());
                } else {
                    e.set_permanent_selected(true);
                }
            });
        }
    } else if let Some(entry) = app_state
        .state
        .tab_state_mut()
        .curr_tab_mut()
        .curr_list_mut()
        .and_then(|s| s.curr_entry_mut())
    {
        if options.reverse {
            entry.set_permanent_selected(false);
        } else if options.toggle {
            entry.set_permanent_selected(!entry.is_selected());
        } else {
            entry.set_permanent_selected(true);
        }
        cursor_move::down(app_state, 1)?;
    }
    Ok(())
}

fn select_with_pattern(
    app_state: &mut AppState,
    pattern: &MatchState,
    options: &SelectOption,
) -> AppResult {
    if let Some(curr_list) = app_state
        .state
        .tab_state_mut()
        .curr_tab_mut()
        .curr_list_mut()
    {
        let mut found = 0;
        curr_list
            .iter_mut()
            .filter(|e| pattern.is_match(e.file_name()))
            .for_each(|e| {
                found += 1;
                if options.reverse {
                    e.set_permanent_selected(false);
                } else if options.toggle {
                    e.set_permanent_selected(!e.is_selected());
                } else {
                    e.set_permanent_selected(true);
                }
            });
        app_state
            .state
            .message_queue_mut()
            .push_info(format!("{} files selected", found));
    }
    Ok(())
}

pub fn select_by_names(
    app_state: &mut AppState,
    names: &HashSet<String>,
    deselect_others: bool,
) -> AppResult {
    Command::new("notify-send")
    .arg("In select - AT START")
    .arg(format!("Calling with {} entries.", names.len()))
    .spawn()
    .expect("Failed to spawn notify-send")
    .wait()
    .expect("notify-send failed");
    if let Some(curr_list) = app_state
        .state
        .tab_state_mut()
        .curr_tab_mut()
        .curr_list_mut()
    {
        Command::new("notify-send")
        .arg("In select")
        .arg(format!("Calling with {} entries.", names.len()))
        .spawn()
        .expect("Failed to spawn notify-send")
        .wait()
        .expect("notify-send failed");
        let mut found = 0;
        let mut deselected = 0;
        curr_list
            .iter_mut()
            .for_each(|e| {
                if names.contains(e.file_name()) {
                    Command::new("notify-send")
                    .arg("In loop -> **FIT FOUND**")
                    .arg(format!("-> {}", e.file_name()))
                    .spawn()
                    .expect("Failed to spawn notify-send")
                    .wait()
                    .expect("notify-send failed");
                    found += 1;
                    e.set_permanent_selected(true);
                } else {
                    if deselect_others && e.is_selected() {
                        deselected += 1;
                        e.set_permanent_selected(false);
                    }
                }
            });
        app_state
            .state
            .message_queue_mut()
            .push_info(format!(
                "{} files selected{}",
                found,
                if deselect_others {format!(", {} files deselected", deselected)} else {String::from("")}
            ));
    }
    Ok(())
}
