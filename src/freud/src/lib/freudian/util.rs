use super::api::ResponseType;

pub fn make_response(code: ResponseType, msg: Option<Vec<u8>>) -> Vec<u8> {
    if msg.is_none() {
        return vec![code.into(), 0, 0];
    }

    let mut msg = msg.unwrap();
    let message_length = msg.len() as u16;
    let mut buffer = vec![code.into(), ((message_length & 0xFF00) >> 8) as u8, (message_length & 0xFF) as u8];
    buffer.append(&mut msg);
    return buffer;
}
