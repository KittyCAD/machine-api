//! The message parser.

use crate::message::Message;

pub(crate) fn parse_message(message: &rumqttc::Event) -> Message {
    match message {
        rumqttc::Event::Incoming(rumqttc::Packet::Publish(publish)) => {
            let payload = publish.payload.clone();

            if let Ok(payload) = std::str::from_utf8(&payload) {
                match serde_json::from_str::<Message>(payload)
                    .map_err(|err| format_serde_error::SerdeError::new(payload.to_string(), err))
                {
                    Ok(message) => {
                        return message;
                    }
                    Err(err) => {
                        tracing::error!("Error parsing message: {:?}", err);
                        if let Ok(message) = serde_json::from_str::<serde_json::Value>(payload) {
                            return Message::Json(message);
                        }
                    }
                }
            }

            Message::Unknown(Some(std::str::from_utf8(&payload).unwrap().to_string()))
        }
        _ => Message::Unknown(None),
    }
}
