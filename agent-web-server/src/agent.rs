use std::str::FromStr;

use anyhow::{Result, anyhow};
use async_openai::types::{
    ChatCompletionRequestAssistantMessageArgs, ChatCompletionRequestMessage,
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use serde::Serialize;
use sqlx::prelude::FromRow;
use strum::{Display, EnumString};

use crate::{
    indoc_info,
    states::{AGENT_CLIENT, SERVER_CONFIG},
};

/// Fire messages to API, returns raw answer (first choice).
pub async fn send_request<I>(messages: I) -> Result<String>
where
    I: IntoIterator<Item = ChatMessage>,
{
    let server_config = SERVER_CONFIG.get().unwrap();
    let sys_message = ChatCompletionRequestSystemMessageArgs::default()
        .content(server_config.sys_prompt.clone())
        .build()?
        .into();

    use MessageRole::*;
    let messages = messages.into_iter().filter_map(|m| match m.get_role() {
        User => ChatCompletionRequestUserMessageArgs::default()
            .content(m.content)
            .build()
            .ok()
            .map(Into::into),
        Assistant => ChatCompletionRequestAssistantMessageArgs::default()
            .content(m.content)
            .build()
            .ok()
            .map(Into::into),
    });

    let complete_messages: Vec<ChatCompletionRequestMessage> =
        std::iter::once(sys_message).chain(messages).collect();

    let request = CreateChatCompletionRequestArgs::default()
        .max_tokens(512u32)
        .model(&server_config.model)
        .messages(complete_messages)
        .build()?;

    let client = AGENT_CLIENT.get().unwrap();
    let response = client.chat().create(request).await?;
    if let Some(ref usage) = response.usage {
        indoc_info!("consumed {} tokens", usage.total_tokens);
    }
    let Some(reply) = response.choices.first() else {
        let resp_json = serde_json::to_string_pretty(&response)
            .unwrap_or("cannot parse response to json".into());
        return Err(anyhow!("no choice in response, response: {}\n", resp_json));
    };
    let Some(ref content) = reply.message.content else {
        let reason = reply.finish_reason;
        return Err(anyhow!(
            "no content in first choice, finish reason: {:?}\n",
            reason
        ));
    };
    Ok(content.clone())
}

#[derive(EnumString, Display)]
pub enum MessageRole {
    User,
    Assistant,
}

#[derive(FromRow, Debug, Serialize)]
pub struct ChatMessage {
    #[serde(skip_serializing)]
    pub uuid: String,
    #[sqlx(rename = "message")]
    pub content: String,
    // sqlx does not support deserialize to enum
    pub role: String,
}

impl ChatMessage {
    pub fn create_user(uuid: &str, content: &str) -> Self {
        Self::create(uuid, content, MessageRole::User)
    }

    pub fn create_assistant(uuid: &str, content: &str) -> Self {
        Self::create(uuid, content, MessageRole::Assistant)
    }

    fn create(uuid: &str, content: &str, role: MessageRole) -> Self {
        Self {
            uuid: uuid.to_string(),
            content: content.to_string(),
            role: role.to_string(),
        }
    }

    pub fn get_role(&self) -> MessageRole {
        match MessageRole::from_str(&self.role) {
            Ok(m) => m,
            Err(_) => {
                unreachable!()
            }
        }
    }
}

#[allow(unused)]
mod test {
    use super::*;

    #[test]
    fn enum_convert_string() {
        let msg = ChatMessage::create_assistant("abc", "def");
        println!("{:?}", msg);
    }
}
