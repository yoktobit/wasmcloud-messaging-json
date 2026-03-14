use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::bindings::wasmcloud::messaging::{consumer, types::BrokerMessage};

mod bindings;

pub fn request<T: Serialize, U: for<'a> Deserialize<'a>>(
    subject: &str,
    body: &T,
    timeout: u32,
) -> Result<U, String> {
    let request_body = serde_json::to_vec(body).map_err(|e| e.to_string())?;
    let msg = bindings::wasmcloud::messaging::consumer::request(subject, &request_body, timeout)?;
    let response: U = serde_json::from_slice(msg.body.as_slice()).map_err(|e| e.to_string())?;
    return Ok(response);
}

pub fn handle<T, U, F>(msg: BrokerMessage, handler: F) -> Result<(), String>
where
    T: DeserializeOwned,
    U: Serialize,
    F: FnOnce(T) -> Result<U, String>,
{
    let Some(subject) = msg.reply_to else {
        return Err("missing reply_to".to_string());
    };

    let request: T = serde_json::from_slice(msg.body.as_slice())
        .map_err(|e| format!("Failed to decode JSON message: {}", e))?;
    let response = handler(request)?;

    publish_json_reply(&subject, &response)
}

fn publish_json_reply<T: Serialize>(subject: &str, response: &T) -> Result<(), String> {
    let body = serde_json::to_vec(response)
        .map_err(|e| format!("Failed to encode JSON response: {}", e))?;

    let reply = BrokerMessage {
        subject: subject.to_string(),
        body: body.into(),
        reply_to: None,
    };

    consumer::publish(&reply)
}
