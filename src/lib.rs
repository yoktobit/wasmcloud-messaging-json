use serde::{Deserialize, Serialize, de::DeserializeOwned};

use crate::bindings::wasmcloud::messaging::{consumer, types::BrokerMessage};

pub mod bindings;

#[allow(unused_macros)]
#[doc(hidden)]
#[macro_export]
macro_rules! export {
  ($ty:ident) => ( crate::bindings::exports::wasmcloud::messaging::handler::__export_wasmcloud_messaging_handler_0_2_0_cabi!($ty with_types_in crate::bindings::exports::wasmcloud::messaging::handler); );
  ($ty:ident with_types_in $($path_to_types_root:tt)*) => (
  $($path_to_types_root)*::exports::wasmcloud::messaging::handler::__export_wasmcloud_messaging_handler_0_2_0_cabi!($ty with_types_in $($path_to_types_root)*::exports::wasmcloud::messaging::handler);
  )
}

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
