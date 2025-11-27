use glam::DVec3;
use std::fmt;

/// Represents a dynamic OpenSCAD value.
/// OpenSCAD is dynamically typed with these supported types.
#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Undef,
    Boolean(bool),
    Number(f64),
    String(String),
    Vector(Vec<Value>),
    Range { start: f64, step: f64, end: f64 },
}

impl Value {
    /// Returns true if the value is "truthy" in OpenSCAD.
    /// - Boolean(true) -> true
    /// - Number != 0 -> true
    /// - String non-empty -> true
    /// - Vector non-empty -> true
    /// - Otherwise -> false
    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Boolean(b) => *b,
            Value::Number(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Vector(v) => !v.is_empty(),
            Value::Range { .. } => true, // Ranges are implicitly true? verify
            Value::Undef => false,
        }
    }

    /// Converts the value to a float (f64).
    /// - Number -> n
    /// - Boolean(true) -> 1.0
    /// - Boolean(false) -> 0.0
    /// - String -> parse or 0.0 (OpenSCAD returns NaN usually for invalid parse, but let's check)
    /// - Undef -> NaN
    pub fn as_f64(&self) -> Option<f64> {
        match self {
            Value::Number(n) => Some(*n),
            Value::Boolean(b) => Some(if *b { 1.0 } else { 0.0 }),
            Value::String(s) => s.parse().ok(), // Simple parse
            Value::Vector(_) => None, // Vector is not a number
            Value::Range { .. } => None,
            Value::Undef => None,
        }
    }
    
    /// Converts to DVec3 if possible.
    pub fn as_vec3(&self) -> Option<DVec3> {
        match self {
            Value::Vector(items) => {
                let x = items.get(0).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let y = items.get(1).and_then(|v| v.as_f64()).unwrap_or(0.0);
                let z = items.get(2).and_then(|v| v.as_f64()).unwrap_or(0.0);
                Some(DVec3::new(x, y, z))
            }
            Value::Number(n) => Some(DVec3::splat(*n)),
            _ => None,
        }
    }
}

impl From<f64> for Value {
    fn from(n: f64) -> Self {
        Value::Number(n)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Value::Boolean(b)
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Value::String(s)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Value::String(s.to_string())
    }
}

impl From<Vec<Value>> for Value {
    fn from(v: Vec<Value>) -> Self {
        Value::Vector(v)
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Undef => write!(f, "undef"),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Number(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Vector(v) => {
                write!(f, "[")?;
                for (i, item) in v.iter().enumerate() {
                    if i > 0 { write!(f, ", ")?; }
                    write!(f, "{}", item)?;
                }
                write!(f, "]")
            }
            Value::Range { start, step, end } => write!(f, "[{}:{}:{}]", start, step, end),
        }
    }
}
