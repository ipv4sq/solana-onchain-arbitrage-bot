use serde::Serialize;
use serde_json::Value;
use solana_sdk::pubkey::Pubkey;

pub fn to_json_value<T: Serialize>(value: &T) -> Value {
    let raw = serde_json::to_value(value).unwrap_or(serde_json::json!(null));
    convert_pubkeys_in_value(raw)
}

fn convert_pubkeys_in_value(value: Value) -> Value {
    match value {
        Value::Object(mut map) => {
            for (_, v) in map.iter_mut() {
                *v = convert_pubkeys_in_value(v.clone());
            }
            Value::Object(map)
        }
        Value::Array(arr) => {
            if is_pubkey_byte_array(&arr) {
                if let Some(pubkey_str) = try_convert_byte_array_to_pubkey(&arr) {
                    return Value::String(pubkey_str);
                }
            }
            Value::Array(arr.into_iter().map(convert_pubkeys_in_value).collect())
        }
        other => other,
    }
}

fn is_pubkey_byte_array(arr: &[Value]) -> bool {
    arr.len() == 32
        && arr
            .iter()
            .all(|v| v.is_u64() && v.as_u64().unwrap_or(256) < 256)
}

fn try_convert_byte_array_to_pubkey(arr: &[Value]) -> Option<String> {
    if arr.len() != 32 {
        return None;
    }

    let mut bytes = [0u8; 32];
    for (i, v) in arr.iter().enumerate() {
        bytes[i] = v.as_u64()? as u8;
    }

    Some(Pubkey::from(bytes).to_string())
}