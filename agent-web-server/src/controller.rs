use crate::{
    agent::{self, ChatMessage},
    auth::{AuthReq, JwtClaim, gen_jwt},
    indoc_debug, indoc_info, indoc_warn,
    protocol::AppResp,
    store,
};
use axum::Json;
use serde::{Deserialize, Serialize};

// Util

type JsonResp<T> = Json<AppResp<T>>;

fn ok<T: Serialize>(resp: T) -> JsonResp<T> {
    Json(AppResp::Success(resp))
}

fn err<T: Serialize>(err: impl Into<String>) -> JsonResp<T> {
    Json(AppResp::Exception(err.into()))
}

// API

pub async fn init_session() -> JsonResp<String> {
    let uuid = uuid::Uuid::new_v4().to_string();
    let jwt = gen_jwt(JwtClaim { uuid });
    ok(jwt)
}

pub async fn fetch_history(req: AuthReq<()>) -> JsonResp<Vec<ChatMessage>> {
    let uuid = req.claim.uuid;
    let history = ChatMessage::load_all(&uuid).await;
    ok(history)
}

pub async fn clear_history(req: AuthReq<()>) -> JsonResp<()> {
    let uuid = req.claim.uuid;
    store::clear_history_by_uuid(&uuid).await;
    ok(())
}

#[derive(Deserialize)]
pub struct AskAgentReq {
    message: String,
}
pub async fn ask_agent(req: AuthReq<AskAgentReq>) -> JsonResp<()> {
    indoc_debug!(
        "
        ip: {:?}
        uuid: {}
        message: {}
        ",
        req.ip,
        req.claim.uuid,
        req.body.message
    );
    let uuid = req.claim.uuid;
    let msg = req.body.message;
    let query_message = ChatMessage::create_user(&uuid, &msg);
    query_message.persist().await;
    let mut history = ChatMessage::load_all(&uuid).await;
    history.push(query_message);
    match agent::send_request(history).await {
        Ok(content) => {
            let reply_message = ChatMessage::create_assistant(&uuid, &content);
            reply_message.persist().await;
            ok(())
        }
        Err(e) => {
            indoc_warn!("Agent Error: {e}");
            err(e.to_string())
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct TestBody {
    id: usize,
    message: String,
}

pub async fn test_auth(req: AuthReq<TestBody>) -> JsonResp<TestBody> {
    indoc_info!(
        "
        Test auth:
        {:?}
        ",
        req
    );
    ok(req.body)
}
