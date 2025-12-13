//! ID intermediate type for compatibility between servers that have different ID formats.
use crate::query::{Arg, IntoArg};

/// ID type used by various Subsonic entities.
#[derive(Debug, Deserialize, Clone, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum Id {
    /// Numeric ID type.
    Numeric(usize),
    /// String ID type.
    String(String),
}

impl IntoArg for Id {
    fn into_arg(self) -> Arg {
        match self {
            Id::Numeric(n) => n.into_arg(),
            Id::String(s) => s.into_arg(),
        }
    }
}

impl std::str::FromStr for Id {
    type Err = std::convert::Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // It's not worth trying to distinguish between numeric and string IDs here.
        // Play it safe, if the user wants a numeric ID they can explicitly convert it.
        Ok(Id::String(s.to_string()))
    }
}

impl std::fmt::Display for Id {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Id::Numeric(n) => write!(f, "{}", n),
            Id::String(s) => write!(f, "{}", s),
        }
    }
}

impl From<usize> for Id {
    fn from(id: usize) -> Self {
        Id::Numeric(id)
    }
}

impl From<String> for Id {
    fn from(id: String) -> Self {
        Id::String(id)
    }
}

impl From<&str> for Id {
    fn from(id: &str) -> Self {
        Id::String(id.to_string())
    }
}
