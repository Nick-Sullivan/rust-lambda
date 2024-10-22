use crate::requests;
use domain::errors::LogicError;
use service;

pub enum RequestType {
    Connect(requests::CreateConnectionRequest),
    CreateGame(requests::CreateGameRequest),
    CreateSession(requests::CreateSessionRequest),
    Disconnect(requests::DestroyConnectionRequest),
    NewRound(requests::NewRoundRequest),
    SetNickname(requests::SetNicknameRequest),
    SetSession(requests::SetSessionRequest),
}

pub fn get_request_type(route_key: &str, body_str: &str) -> Result<RequestType, LogicError> {
    if route_key == "$connect" {
        return Ok(RequestType::Connect(requests::CreateConnectionRequest {}));
    }
    if route_key == "$disconnect" {
        return Ok(RequestType::Disconnect(
            requests::DestroyConnectionRequest {},
        ));
    }

    let request: requests::WebsocketRequest = serde_json::from_str(&body_str)
        .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
    println!("Request action {}", request.action);
    println!("Request data {}", request.data);
    match request.action.as_str() {
        "createGame" => {
            let request: requests::CreateGameRequest = serde_json::from_value(request.data)
                .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
            Ok(RequestType::CreateGame(request))
        }
        "getSession" => {
            let request: requests::CreateSessionRequest = serde_json::from_value(request.data)
                .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
            Ok(RequestType::CreateSession(request))
        }
        "newRound" => {
            let request: requests::NewRoundRequest = serde_json::from_value(request.data)
                .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
            Ok(RequestType::NewRound(request))
        }
        "setNickname" => {
            let request: requests::SetNicknameRequest = serde_json::from_value(request.data)
                .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
            Ok(RequestType::SetNickname(request))
        }
        "setSession" => {
            let request: requests::SetSessionRequest = serde_json::from_value(request.data)
                .map_err(|e| LogicError::DeserializationError(e.to_string()))?;
            Ok(RequestType::SetSession(request))
        }
        _ => Err(LogicError::WebsocketError("Unknown action".to_string()))?,
    }
}

pub async fn route(request_type: &RequestType, connection_id: &str) -> Result<String, LogicError> {
    match request_type {
        RequestType::Connect(request) => {
            let command = request.to_command(connection_id);
            service::create_connection::handler(&command).await
        }
        RequestType::CreateGame(request) => {
            let command = request.to_command(connection_id);
            service::create_game::handler(&command).await
        }
        RequestType::CreateSession(request) => {
            let command = request.to_command(connection_id);
            service::create_session::handler(&command).await
        }
        RequestType::Disconnect(request) => {
            let command = request.to_command(connection_id);
            service::destroy_connection::handler(&command).await
        }
        RequestType::NewRound(request) => {
            let command = request.to_command(connection_id);
            service::new_round::handler(&command).await
        }
        RequestType::SetNickname(request) => {
            let command = request.to_command(connection_id);
            service::set_nickname::handler(&command).await
        }
        RequestType::SetSession(request) => {
            let command = request.to_command(connection_id);
            service::set_session::handler(&command).await
        }
    }
}
