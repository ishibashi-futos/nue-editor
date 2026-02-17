use nue_core::command_hub_actions::{
    CommandActionError, CommandActionEvent, CommandHubDispatchOutcome,
};

/// Command Hub の UI で必要なオーバーレイ操作を表現する。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandHubOverlayAction {
    /// コマンドパレットを継続表示する。
    KeepOpen,
    /// コマンドパレットを閉じる。
    CloseOverlay { candidate_id: Option<String> },
    /// 候補一覧へ戻る。
    ReturnToListing { candidate_id: Option<String> },
    /// 確認待ち状態に移行する。
    NeedsConfirmation { candidate_id: String },
}

/// 通知の重要度を示す。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CommandHubNotificationLevel {
    Info,
    Warning,
    Error,
}

/// Command Hub で表示する通知。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandHubNotification {
    pub level: CommandHubNotificationLevel,
    pub message: String,
    pub candidate_id: Option<String>,
}

/// Command Hub の結果を受けて UI に命令を与える構造体。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandHubUiTransition {
    pub overlay_action: CommandHubOverlayAction,
    pub notification: Option<CommandHubNotification>,
    pub executed_event: Option<CommandActionEvent>,
}

impl CommandHubUiTransition {
    /// Dispatch の結果から UI 制御命令を生成する。
    pub fn from_outcome(outcome: CommandHubDispatchOutcome) -> Self {
        match outcome {
            CommandHubDispatchOutcome::NoSelection => Self {
                overlay_action: CommandHubOverlayAction::CloseOverlay { candidate_id: None },
                notification: Some(CommandHubNotification {
                    level: CommandHubNotificationLevel::Warning,
                    message: "候補を選ばずにコマンドパレットを閉じました。".to_string(),
                    candidate_id: None,
                }),
                executed_event: None,
            },
            CommandHubDispatchOutcome::NeedsConfirmation { candidate_id } => Self {
                overlay_action: CommandHubOverlayAction::NeedsConfirmation { candidate_id },
                notification: None,
                executed_event: None,
            },
            CommandHubDispatchOutcome::Executed(event) => Self {
                overlay_action: CommandHubOverlayAction::CloseOverlay { candidate_id: None },
                notification: Some(CommandHubNotification {
                    level: CommandHubNotificationLevel::Info,
                    message: "コマンドを実行しました。".to_string(),
                    candidate_id: None,
                }),
                executed_event: Some(event),
            },
            CommandHubDispatchOutcome::Failed(error) => Self {
                overlay_action: CommandHubOverlayAction::ReturnToListing { candidate_id: None },
                notification: Some(CommandHubNotification {
                    level: CommandHubNotificationLevel::Error,
                    message: message_for_error(&error),
                    candidate_id: None,
                }),
                executed_event: None,
            },
            CommandHubDispatchOutcome::Closed { candidate_id } => {
                let notification_candidate = candidate_id.clone();
                Self {
                    overlay_action: CommandHubOverlayAction::CloseOverlay {
                        candidate_id: candidate_id.clone(),
                    },
                    notification: Some(CommandHubNotification {
                        level: CommandHubNotificationLevel::Info,
                        message: "コマンドパレットを閉じました。".to_string(),
                        candidate_id: notification_candidate,
                    }),
                    executed_event: None,
                }
            }
            CommandHubDispatchOutcome::BackToListing { candidate_id } => {
                let notification_candidate = candidate_id.clone();
                Self {
                    overlay_action: CommandHubOverlayAction::ReturnToListing {
                        candidate_id: candidate_id.clone(),
                    },
                    notification: Some(CommandHubNotification {
                        level: CommandHubNotificationLevel::Info,
                        message: "候補一覧に戻しました。".to_string(),
                        candidate_id: notification_candidate,
                    }),
                    executed_event: None,
                }
            }
        }
    }
}

fn message_for_error(error: &CommandActionError) -> String {
    match error {
        CommandActionError::UnsupportedCommand { domain, verb } => {
            format!("{} {} は未対応のコマンドです。", domain, verb)
        }
        CommandActionError::MissingTarget { domain, verb } => {
            format!("{} {} の対象が指定されていません。", domain, verb)
        }
        CommandActionError::NotFound { resource, target } => {
            format!("{} '{}' が見つかりませんでした。", resource, target)
        }
        CommandActionError::NoSelection => "候補が選択されませんでした。".to_string(),
        CommandActionError::InvalidTarget { reason } => {
            format!("対象が無効です：{}。", reason)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nue_core::command_hub_actions::{CommandActionError, CommandHubDispatchOutcome};

    #[test]
    fn closedはオーバーレイを閉じ通知する() {
        let transition = CommandHubUiTransition::from_outcome(CommandHubDispatchOutcome::Closed {
            candidate_id: Some("cand".into()),
        });

        assert_eq!(
            transition.overlay_action,
            CommandHubOverlayAction::CloseOverlay {
                candidate_id: Some("cand".into())
            }
        );
        let notification = transition.notification.expect("通知があるはず");
        assert_eq!(notification.level, CommandHubNotificationLevel::Info);
        assert!(notification.message.contains("閉じました"));
        assert_eq!(notification.candidate_id, Some("cand".into()));
    }

    #[test]
    fn back_to_listingは一覧復帰アクションを返す() {
        let transition =
            CommandHubUiTransition::from_outcome(CommandHubDispatchOutcome::BackToListing {
                candidate_id: None,
            });

        assert_eq!(
            transition.overlay_action,
            CommandHubOverlayAction::ReturnToListing { candidate_id: None }
        );
        assert_eq!(
            transition.notification.unwrap().level,
            CommandHubNotificationLevel::Info
        );
    }

    #[test]
    fn failedはエラー通知を含む() {
        let transition = CommandHubUiTransition::from_outcome(CommandHubDispatchOutcome::Failed(
            CommandActionError::UnsupportedCommand {
                domain: "tab".into(),
                verb: "close".into(),
            },
        ));

        assert_eq!(
            transition.overlay_action,
            CommandHubOverlayAction::ReturnToListing { candidate_id: None }
        );
        let notification = transition.notification.expect("エラー通知があるはず");
        assert_eq!(notification.level, CommandHubNotificationLevel::Error);
        assert!(notification.message.contains("未対応"));
    }

    #[test]
    fn no_selectionはキャンセル扱いで閉じる() {
        let transition =
            CommandHubUiTransition::from_outcome(CommandHubDispatchOutcome::NoSelection);
        assert_eq!(
            transition.overlay_action,
            CommandHubOverlayAction::CloseOverlay { candidate_id: None }
        );
        let notification = transition.notification.expect("警告通知があるはず");
        assert_eq!(notification.level, CommandHubNotificationLevel::Warning);
        assert!(notification.message.contains("候補"));
    }

    #[test]
    fn needs_confirmationは確認アクションになる() {
        let transition =
            CommandHubUiTransition::from_outcome(CommandHubDispatchOutcome::NeedsConfirmation {
                candidate_id: "confirm".into(),
            });
        assert_eq!(
            transition.overlay_action,
            CommandHubOverlayAction::NeedsConfirmation {
                candidate_id: "confirm".into()
            }
        );
        assert!(transition.notification.is_none());
    }
}
