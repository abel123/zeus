use serde::{Serialize, Serializer};

#[derive(PartialEq, Debug, Clone)]
pub enum Direction {
    Up,
    Down,
}

impl Direction {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Up => "向上",
            Self::Down => "向下",
        }
    }
}
impl Serialize for Direction {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            Self::Up => serializer.serialize_str("up"),
            Self::Down => serializer.serialize_str("down"),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Mark {
    D,
    G,
}

impl Mark {
    fn as_str(&self) -> &'static str {
        match self {
            Self::D => "底分型",
            Self::G => "顶分型",
        }
    }
}

#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, Serialize)]
pub enum Freq {
    Tick,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F10,
    F12,
    F15,
    F20,
    F30,
    F60,
    F120,
    F240,
    F480,
    D,
    W,
    M,
    S,
    Y,
}

impl Freq {
    fn as_str(&self) -> &'static str {
        match self {
            Freq::Tick => "Tick",
            Freq::F1 => "F1",
            Freq::F2 => "F2",
            Freq::F3 => "F3",
            Freq::F4 => "F4",
            Freq::F5 => "F5",
            Freq::F6 => "F6",
            Freq::F10 => "F10",
            Freq::F12 => "F12",
            Freq::F15 => "F15",
            Freq::F20 => "F20",
            Freq::F30 => "F30",
            Freq::F60 => "F60",
            Freq::F120 => "F120",
            Freq::F240 => "F240",
            Freq::F480 => "F480",
            Freq::D => "D",
            Freq::W => "W",
            Freq::M => "M",
            Freq::S => "S", //季线
            Freq::Y => "Y",
        }
    }
}
