use serde::{Deserialize, Deserializer, Serialize};

// ---------------------------------------------------------------------------
// Color
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

/// Deserialize a hex color string (`#RRGGBB[AA]`) into a `Color`.
fn color_from_hex<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    Ok(Color::from_hex(&s))
}

/// Deserialize an optional hex color string into `Option<Color>`.
fn opt_color_from_hex<'de, D>(deserializer: D) -> Result<Option<Color>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(deserializer)?;
    Ok(s.map(|s| Color::from_hex(&s)))
}

impl Default for Color {
    fn default() -> Self {
        // Transparent black — sensible fallback for a missing color field.
        Color {
            r: 0,
            g: 0,
            b: 0,
            a: 0,
        }
    }
}

impl Color {
    /// Parse `#RRGGBB` or `#RRGGBBAA`. Panics on malformed input.
    pub fn from_hex(s: &str) -> Self {
        let s = s.strip_prefix('#').unwrap_or(s);
        let bytes = match s.len() {
            6 => {
                let mut b = [0u8; 4];
                let pairs = hex_pairs(s);
                b[0] = pairs[0];
                b[1] = pairs[1];
                b[2] = pairs[2];
                b[3] = 255;
                b
            }
            8 => {
                let mut b = [0u8; 4];
                let pairs = hex_pairs(s);
                b[0] = pairs[0];
                b[1] = pairs[1];
                b[2] = pairs[2];
                b[3] = pairs[3];
                b
            }
            _ => panic!("invalid color hex: '{s}' (expected #RRGGBB or #RRGGBBAA)"),
        };
        Color {
            r: bytes[0],
            g: bytes[1],
            b: bytes[2],
            a: bytes[3],
        }
    }

    pub fn to_hex(&self) -> String {
        format!("#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }

    /// Premultiply? No — straight (non-premultiplied) RGBA. Blend math handles alpha.
    pub fn rgba(self) -> [u8; 4] {
        [self.r, self.g, self.b, self.a]
    }
}

fn hex_pairs(s: &str) -> [u8; 4] {
    fn val(c: u8) -> u8 {
        match c {
            b'0'..=b'9' => c - b'0',
            b'a'..=b'f' => c - b'a' + 10,
            b'A'..=b'F' => c - b'A' + 10,
            _ => panic!("invalid hex digit: '{}'", c as char),
        }
    }
    let b = s.as_bytes();
    [
        (val(b[0]) << 4) | val(b[1]),
        (val(b[2]) << 4) | val(b[3]),
        (val(b[4]) << 4) | val(b[5]),
        if b.len() >= 8 {
            (val(b[6]) << 4) | val(b[7])
        } else {
            255
        },
    ]
}

// ---------------------------------------------------------------------------
// Transform
// ---------------------------------------------------------------------------

fn default_pos() -> [f32; 2] {
    [0.0, 0.0]
}
fn default_scale() -> [f32; 2] {
    [1.0, 1.0]
}
fn default_opacity() -> f32 {
    100.0
}
fn default_rotation() -> f32 {
    0.0
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub struct Transform {
    #[serde(default = "default_pos")]
    pub position: [f32; 2],
    #[serde(default = "default_scale")]
    pub scale: [f32; 2],
    #[serde(default = "default_opacity")]
    pub opacity: f32,
    #[serde(default = "default_rotation")]
    pub rotation: f32,
}

impl Default for Transform {
    fn default() -> Self {
        Transform {
            position: default_pos(),
            scale: default_scale(),
            opacity: default_opacity(),
            rotation: default_rotation(),
        }
    }
}

// ---------------------------------------------------------------------------
// Blend mode
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BlendMode {
    Normal,
}

// ---------------------------------------------------------------------------
// Animation: keyframes + expressions
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AnimProp {
    Position,
    Scale,
    Opacity,
    Rotation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Easing {
    #[serde(rename = "linear")]
    Linear,
    #[serde(rename = "ease_in")]
    EaseIn,
    #[serde(rename = "ease_out")]
    EaseOut,
    #[serde(rename = "ease_in_out")]
    EaseInOut,
    #[serde(rename = "step")]
    Step,
}

fn default_easing() -> Easing {
    Easing::Linear
}

/// A keyframe time: a frame number or a "MM:SS:FF" timecode (resolved via project fps).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum KeyTime {
    Frame(u32),
    Tc(String),
}

/// A per-keyframe value: a scalar (opacity/rotation) or 2-vector (position/scale).
/// Custom deserialize accepts TOML int OR float scalars and [x, y] arrays.
#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
pub enum AnimValue {
    Number(f32),
    Vec2([f32; 2]),
}

fn anim_value_deser<'de, D>(d: D) -> Result<AnimValue, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let v = toml::Value::deserialize(d)?;
    match v {
        toml::Value::Integer(i) => Ok(AnimValue::Number(i as f32)),
        toml::Value::Float(f) => Ok(AnimValue::Number(f as f32)),
        toml::Value::Array(a) if a.len() == 2 => {
            let mut out = [0.0f32; 2];
            for (i, item) in a.into_iter().enumerate() {
                out[i] = match item {
                    toml::Value::Integer(x) => x as f32,
                    toml::Value::Float(x) => x as f32,
                    _ => {
                        return Err(serde::de::Error::custom(
                            "keyframe value must be a number or [x, y]",
                        ))
                    }
                };
            }
            Ok(AnimValue::Vec2(out))
        }
        _ => Err(serde::de::Error::custom(
            "keyframe value must be a number or [x, y]",
        )),
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Keyframe {
    pub property: AnimProp,
    pub time: KeyTime,
    #[serde(deserialize_with = "anim_value_deser")]
    pub value: AnimValue,
    #[serde(default = "default_easing")]
    pub easing: Easing,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Expression {
    pub property: AnimProp,
    pub expr: String,
}

// ---------------------------------------------------------------------------
// Layer
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LayerKind {
    Solid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Layer {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: LayerKind,
    #[serde(default, deserialize_with = "opt_color_from_hex")]
    pub color: Option<Color>,
    #[serde(default)]
    pub size: [f32; 2], // [w, h]; [0, 0] = full canvas
    #[serde(default)]
    pub transform: Transform,
    #[serde(rename = "keyframe", default)]
    pub keyframes: Vec<Keyframe>,
    #[serde(rename = "expression", default)]
    pub expressions: Vec<Expression>,
    #[serde(default = "default_blend")]
    pub blend: BlendMode,
}

fn default_blend() -> BlendMode {
    BlendMode::Normal
}

// ---------------------------------------------------------------------------
// Project
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Project {
    pub project: ProjectInfo,
    #[serde(rename = "layer", default)]
    pub layers: Vec<Layer>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ProjectInfo {
    pub name: String,
    pub fps: u32,
    pub width: u32,
    pub height: u32,
    pub duration: u32,
    #[serde(rename = "bg_color", deserialize_with = "color_from_hex")]
    pub bg_color: Color,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_rgb() {
        let c = Color::from_hex("#ff0000");
        assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 255));
    }

    #[test]
    fn color_rgba() {
        let c = Color::from_hex("#ff000080");
        assert_eq!((c.r, c.g, c.b, c.a), (255, 0, 0, 128));
    }

    #[test]
    #[should_panic]
    fn color_invalid_panics() {
        let _ = Color::from_hex("#zzz");
    }
}
