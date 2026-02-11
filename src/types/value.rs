#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Null,
    Int(i64),
    String(Vec<u8>),
    Array(Vec<Value>),
    Map(Vec<(Value, Value)>),
    Set(Vec<Value>),
    Bool(bool),
    Okay,
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s.into_bytes())
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.as_bytes().to_vec())
    }
}

impl From<Vec<u8>> for Value {
    fn from(b: Vec<u8>) -> Self {
        Value::String(b)
    }
}

impl From<i64> for Value {
    fn from(n: i64) -> Self {
        Value::Int(n)
    }
}

impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Value::Int(n as i64)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Bool(b)
    }
}

impl From<()> for Value {
    fn from(_: ()) -> Self {
        Value::Null
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Array(v)
    }
}
