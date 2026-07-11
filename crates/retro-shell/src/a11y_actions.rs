//! Accessible actions for shell chrome (AT-SPI Action interface pure map).
//!
//! Maps focus targets and common widgets to named actions an AT can invoke.
//! Not a full Orca stack — provides stable action names + invoke plans.

use crate::chrome_protocol::ChromeFocusTarget;
use crate::session_actions::SessionAction;

/// Named accessible action (AT-SPI Action.name style).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AccessibleAction {
    pub name: String,
    pub description: String,
    /// Shell menu / session action id when applicable.
    pub invoke_id: String,
}

/// Pure: actions available for a chrome focus region.
pub fn actions_for_chrome(target: ChromeFocusTarget) -> Vec<AccessibleAction> {
    match target {
        ChromeFocusTarget::MenuBar => vec![
            AccessibleAction {
                name: "click".into(),
                description: "Open the focused menu".into(),
                invoke_id: "chrome.menu.activate".into(),
            },
            AccessibleAction {
                name: "show-menu".into(),
                description: "Show the Retro system menu".into(),
                invoke_id: "chrome.menu.system".into(),
            },
        ],
        ChromeFocusTarget::Dock => vec![
            AccessibleAction {
                name: "click".into(),
                description: "Launch or focus the dock item".into(),
                invoke_id: "chrome.dock.activate".into(),
            },
            AccessibleAction {
                name: "show-menu".into(),
                description: "Show dock item context menu".into(),
                invoke_id: "chrome.dock.menu".into(),
            },
        ],
        ChromeFocusTarget::DesktopIcons => vec![
            AccessibleAction {
                name: "click".into(),
                description: "Open the selected desktop icon".into(),
                invoke_id: "chrome.desktop.open".into(),
            },
            AccessibleAction {
                name: "show-menu".into(),
                description: "Show desktop context menu".into(),
                invoke_id: "chrome.desktop.menu".into(),
            },
        ],
        ChromeFocusTarget::Windows => vec![
            AccessibleAction {
                name: "activate".into(),
                description: "Raise and focus the window".into(),
                invoke_id: "chrome.window.activate".into(),
            },
            AccessibleAction {
                name: "close".into(),
                description: "Close the focused window".into(),
                invoke_id: "chrome.window.close".into(),
            },
            AccessibleAction {
                name: "minimize".into(),
                description: "Minimize the focused window".into(),
                invoke_id: "chrome.window.minimize".into(),
            },
        ],
    }
}

/// Session-level accessible actions always available from the shell root.
pub fn session_root_actions() -> Vec<AccessibleAction> {
    vec![
        AccessibleAction {
            name: "lock-screen".into(),
            description: "Lock the session".into(),
            invoke_id: "shell.lock".into(),
        },
        AccessibleAction {
            name: "log-out".into(),
            description: "Log out of the session".into(),
            invoke_id: "shell.log_out".into(),
        },
        AccessibleAction {
            name: "notification-center".into(),
            description: "Open the notification center".into(),
            invoke_id: "shell.notification_center".into(),
        },
        AccessibleAction {
            name: "force-quit".into(),
            description: "Open Force Quit".into(),
            invoke_id: "shell.force_quit".into(),
        },
    ]
}

/// Map an accessible action invoke_id to a session action when applicable.
pub fn session_action_for_invoke(invoke_id: &str) -> Option<SessionAction> {
    SessionAction::from_menu_action(invoke_id)
}

/// AT-SPI Action interface summary for a path (tests / D-Bus serialize).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActionInterfaceSummary {
    pub path: String,
    pub n_actions: i32,
    pub names: Vec<String>,
    pub descriptions: Vec<String>,
}

pub fn summarize_actions(path: &str, actions: &[AccessibleAction]) -> ActionInterfaceSummary {
    ActionInterfaceSummary {
        path: path.to_string(),
        n_actions: actions.len() as i32,
        names: actions.iter().map(|a| a.name.clone()).collect(),
        descriptions: actions.iter().map(|a| a.description.clone()).collect(),
    }
}

/// DoInvoke plan: which shell handler to call (pure).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct InvokePlan {
    pub invoke_id: String,
    pub valid: bool,
}

pub fn plan_invoke(actions: &[AccessibleAction], index: i32) -> InvokePlan {
    if index < 0 {
        return InvokePlan {
            invoke_id: String::new(),
            valid: false,
        };
    }
    match actions.get(index as usize) {
        Some(a) => InvokePlan {
            invoke_id: a.invoke_id.clone(),
            valid: true,
        },
        None => InvokePlan {
            invoke_id: String::new(),
            valid: false,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_and_session_actions() {
        let dock = actions_for_chrome(ChromeFocusTarget::Dock);
        assert!(dock.iter().any(|a| a.name == "click"));
        let root = session_root_actions();
        assert!(root.iter().any(|a| a.invoke_id == "shell.lock"));
        assert_eq!(
            session_action_for_invoke("shell.lock"),
            Some(SessionAction::Lock)
        );
    }

    #[test]
    fn invoke_plan_bounds() {
        let a = actions_for_chrome(ChromeFocusTarget::Windows);
        let p = plan_invoke(&a, 0);
        assert!(p.valid);
        assert!(!plan_invoke(&a, 99).valid);
        let s = summarize_actions("/org/a11y/atspi/accessible/3", &a);
        assert_eq!(s.n_actions, a.len() as i32);
    }
}
