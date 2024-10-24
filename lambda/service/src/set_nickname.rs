use chrono::Utc;
use domain::commands::SetNicknameCommand;
use domain::errors::LogicError;
use notifier::{self, ActionType, INotifier, Message, SetNicknameMessage};
use std::collections::HashSet;
use storage::session_table::{SessionAction, SessionItem};
use storage::IDynamoDbClient;

pub async fn handler(command: &SetNicknameCommand) -> Result<String, LogicError> {
    let db = storage::get().await;
    let notifier = notifier::get().await;

    let mut session = SessionItem::from_db(&command.session_id, &db).await?;

    let is_valid = is_valid_nickname(&command.nickname);
    if is_valid {
        session.account_id = command.account_id.clone();
        session.modified_action = SessionAction::SetNickname;
        session.modified_at = Utc::now();
        session.nickname = Some(command.nickname.clone());
        session.version += 1;

        db.write_single(session.save()?).await?;
        let message = create_success_message(&command.session_id, &command.nickname);
        notifier.notify(&session.connection_id, &message).await?;
    } else {
        let message = create_failure_message();
        notifier.notify(&session.connection_id, &message).await?;
    }
    Ok(command.session_id.clone())
}

fn is_valid_nickname(nickname: &str) -> bool {
    let invalid_names: HashSet<&str> = ["MR ELEVEN", "MRELEVEN", "MR 11", "MR11"]
        .iter()
        .cloned()
        .collect();
    let length = nickname.len();
    let name_upper = nickname.trim().to_uppercase();

    length >= 2 && length <= 69 && !invalid_names.contains(name_upper.as_str())
}

fn create_success_message(session_id: &str, nickname: &str) -> Message {
    let nickname_message = SetNicknameMessage {
        nickname: nickname.to_string(),
        player_id: session_id.to_string(),
    };
    Message::new(ActionType::SetNickname(nickname_message))
}

fn create_failure_message() -> Message {
    Message::new_err(ActionType::SetNicknameFailure(
        "Invalid nickname".to_string(),
    ))
}
