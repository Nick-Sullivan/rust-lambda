use actix::{Actor, StreamHandler};
use actix_web::{web, App, Error, HttpRequest, HttpResponse, HttpServer};
use actix_web_actors::ws;
use api::{
    requests,
    websocket::{self, RequestType},
};
use domain::errors::LogicError;
use std::env;
use uuid::Uuid;

struct MyWs {
    connection_id: String,
}

impl Actor for MyWs {
    type Context = ws::WebsocketContext<Self>;
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env::set_var("WEBSOCKET_TABLE_NAME", "RustLambda-DevWebsocket");
    env::set_var("GAME_TABLE_NAME", "RustLambda-DevGame");
    env::set_var("AWS_REGION", "eu-west-2");
    env::set_var("API_GATEWAY_URL", "ws://127.0.0.1:8080/ws/");
    HttpServer::new(|| App::new().route("/ws/", web::get().to(index)))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}

async fn index(req: HttpRequest, stream: web::Payload) -> Result<HttpResponse, Error> {
    println!("index");
    let connection_id = Uuid::new_v4().to_string();
    let websocket = MyWs {
        connection_id: connection_id.clone(),
    };
    let resp = ws::start(websocket, &req, stream);
    let result = connect(&connection_id).await;
    match result {
        Ok(_) => resp,
        Err(e) => {
            let message = format!("Error: {e}");
            return Err(actix_web::error::ErrorInternalServerError(message));
        }
    }
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for MyWs {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        println!("handle, connection_id: {}", self.connection_id);
        match msg {
            Ok(ws::Message::Text(text)) => {
                println!("text");
                let connection_id = self.connection_id.clone();
                actix::spawn(async move {
                    let result = message(&connection_id, &text).await;
                    match result {
                        Ok(_) => (),
                        Err(e) => println!("Error disconnecting: {e}"),
                    }
                });
                // ctx.spawn(wrap_future(fut.into_actor(self)));

                // ctx.text(text)
            }
            Ok(ws::Message::Close(reason)) => {
                println!("close");
                let connection_id = self.connection_id.clone();
                actix::spawn(async move {
                    let result = disconnect(&connection_id).await;
                    match result {
                        Ok(_) => (),
                        Err(e) => println!("Error disconnecting: {e}"),
                    }
                });
                ctx.close(reason)
            }
            _ => {
                println!("Recieved unsupported message type");
                ()
            }
        }
    }
}

async fn connect(connection_id: &str) -> Result<String, LogicError> {
    let request_type = RequestType::Connect(requests::CreateConnectionRequest {});
    let result = websocket::route(&request_type, connection_id).await;
    result
}

async fn disconnect(connection_id: &str) -> Result<String, LogicError> {
    let request_type = RequestType::Disconnect(requests::DestroyConnectionRequest {});
    let result = websocket::route(&request_type, &connection_id).await;
    result
}

async fn message(connection_id: &str, text: &str) -> Result<String, LogicError> {
    let request_type = websocket::get_request_type("$default", &text)?;
    let result = websocket::route(&request_type, &connection_id).await;
    result
}
