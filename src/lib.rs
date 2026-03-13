use serde::{Deserialize, Serialize};

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
