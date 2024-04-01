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
            Freq::F1 => "1分钟",
            Freq::F2 => "2分钟",
            Freq::F3 => "3分钟",
            Freq::F4 => "4分钟",
            Freq::F5 => "5分钟",
            Freq::F6 => "6分钟",
            Freq::F10 => "10分钟",
            Freq::F12 => "12分钟",
            Freq::F15 => "15分钟",
            Freq::F20 => "20分钟",
            Freq::F30 => "30分钟",
            Freq::F60 => "60分钟",
            Freq::F120 => "120分钟",
            Freq::F240 => "240分钟",
            Freq::F480 => "480分钟",
            Freq::D => "日线",
            Freq::W => "周线",
            Freq::M => "月线",
            Freq::S => "季线",
            Freq::Y => "年线",
        }
    }
}
