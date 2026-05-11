use serde_json::{Number, Value};

pub fn clean_json_numbers(v: &mut Value) {
    match v {
        Value::Number(n) => {
            if let Some(f) = n.as_f64() {
                if f.fract() == 0.0 {
                    *v = Value::Number(Number::from(f as i64));
                }
            }
        }
        Value::Array(arr) => {
            for item in arr {
                clean_json_numbers(item);
            }
        }
        Value::Object(obj) => {
            for val in obj.values_mut() {
                clean_json_numbers(val);
            }
        }
        _ => {}
    }
}