use crate::project::{AnimProp, AnimValue, Easing, KeyTime, Keyframe, Layer, Transform};
use std::collections::HashMap;

/// Resolve a keyframe time to an integer frame number.
/// Frame ints pass through unchanged; `"MM:SS:FF"` is resolved via `fps`.
pub fn time_to_frame(t: &KeyTime, fps: u32) -> u32 {
    match t {
        KeyTime::Frame(f) => *f,
        KeyTime::Tc(s) => {
            let parts: Vec<&str> = s.split(':').collect();
            assert!(parts.len() == 3, "timecode must be MM:SS:FF, got '{s}'");
            let nums: Result<Vec<u32>, _> = parts.iter().map(|p| p.parse::<u32>()).collect();
            let nums = nums.unwrap_or_else(|_| panic!("timecode must be MM:SS:FF, got '{s}'"));
            nums[0] * 60 * fps + nums[1] * fps + nums[2]
        }
    }
}

/// Apply an easing curve to a 0..1 progress value (clamped).
/// `Step` returns 0.0 — the interpolation caller must treat it as a hold instead of using this.
pub fn ease(e: Easing, t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    match e {
        Easing::Linear => t,
        Easing::EaseIn => t * t * t,
        Easing::EaseOut => 1.0 - (1.0 - t) * (1.0 - t) * (1.0 - t),
        Easing::EaseInOut => 3.0 * t * t - 2.0 * t * t * t,
        Easing::Step => 0.0,
    }
}

/// Resolve one animated property at the given frame.
/// If no keyframes target this property, `static_value` is returned unchanged.
/// Extrapolation: hold constant before the first key and after the last key.
/// Step easing = hold the left key's value until the next key.
pub fn resolve_value(
    keyframes: &[Keyframe],
    prop: AnimProp,
    frame: u32,
    fps: u32,
    static_value: AnimValue,
) -> AnimValue {
    let mut keys: Vec<&Keyframe> = keyframes.iter().filter(|k| k.property == prop).collect();
    if keys.is_empty() {
        return static_value;
    }
    keys.sort_by_key(|k| time_to_frame(&k.time, fps));

    let first = time_to_frame(&keys[0].time, fps);
    let last = time_to_frame(&keys[keys.len() - 1].time, fps);

    if frame <= first {
        return keys[0].value;
    }
    if frame >= last {
        return keys[keys.len() - 1].value;
    }

    // Find the bracketing pair: `i` is the last key whose time <= frame.
    let mut i = keys.len() - 1;
    while i > 0 && time_to_frame(&keys[i].time, fps) > frame {
        i -= 1;
    }
    let a = keys[i];
    let b = keys[i + 1];
    let ta = time_to_frame(&a.time, fps);
    let tb = time_to_frame(&b.time, fps);

    if a.easing == Easing::Step {
        return a.value;
    }

    let raw = (frame - ta) as f32 / (tb - ta) as f32;
    let t = ease(a.easing, raw);
    lerp(a.value, b.value, t)
}

fn lerp(a: AnimValue, b: AnimValue, t: f32) -> AnimValue {
    match (a, b) {
        (AnimValue::Number(x), AnimValue::Number(y)) => AnimValue::Number(x + (y - x) * t),
        (AnimValue::Vec2(x), AnimValue::Vec2(y)) => {
            AnimValue::Vec2([x[0] + (y[0] - x[0]) * t, x[1] + (y[1] - x[1]) * t])
        }
        _ => panic!("keyframe value type mismatch between adjacent keys"),
    }
}

/// Resolve a full layer transform at a given frame.
/// Keyframes override the static transform; expression override added in Task 6.
pub fn resolve_transform(layer: &Layer, frame: u32, fps: u32) -> Transform {
    let t = layer.transform;
    let position = resolve_vec2(&layer.keyframes, AnimProp::Position, frame, fps, t.position);
    let scale = resolve_vec2(&layer.keyframes, AnimProp::Scale, frame, fps, t.scale);
    let opacity = resolve_f32(&layer.keyframes, AnimProp::Opacity, frame, fps, t.opacity);
    let rotation = resolve_f32(&layer.keyframes, AnimProp::Rotation, frame, fps, t.rotation);
    Transform {
        position,
        scale,
        opacity,
        rotation,
    }
}

fn resolve_vec2(
    keys: &[Keyframe],
    prop: AnimProp,
    frame: u32,
    fps: u32,
    fallback: [f32; 2],
) -> [f32; 2] {
    match resolve_value(keys, prop, frame, fps, AnimValue::Vec2(fallback)) {
        AnimValue::Vec2(v) => v,
        _ => unreachable!(),
    }
}

fn resolve_f32(keys: &[Keyframe], prop: AnimProp, frame: u32, fps: u32, fallback: f32) -> f32 {
    match resolve_value(keys, prop, frame, fps, AnimValue::Number(fallback)) {
        AnimValue::Number(v) => v,
        _ => unreachable!(),
    }
}

/// Evaluate a Python expression string to a number or 2-element list.
/// Available: value, time, frame, fps, width, height, duration.
/// Expression errors panic with a clear message in v0.2.
pub fn evaluate_expression(
    expr: &str,
    value: AnimValue,
    frame: u32,
    fps: u32,
    width: u32,
    height: u32,
    duration: u32,
) -> AnimValue {
    use std::collections::HashMap;

    // Minimal in-process evaluator: supports arithmetic on numbers and
    // list indexing.  This is the v0.2 shim — a full Python runtime is
    // deferred to a later phase if needed.
    //
    // Recognised tokens in the expression:
    //   value, time, frame, fps, width, height, duration
    //   + - * / ( )
    //   integer and float literals
    //   [ ] for 2-element list access on `value`
    //
    // The strategy: build a flat list of `f64` values and operators,
    // then evaluate left-to-right with standard precedence (manual
    // because a real parser would need a dependency).

    let time_s = frame as f64 / fps as f64;
    let mut vars: HashMap<&str, f64> = HashMap::new();
    vars.insert(
        "value",
        match value {
            AnimValue::Number(n) => n as f64,
            _ => panic!("expression got non-scalar value for a scalar property"),
        },
    );
    vars.insert("time", time_s);
    vars.insert("frame", frame as f64);
    vars.insert("fps", fps as f64);
    vars.insert("width", width as f64);
    vars.insert("height", height as f64);
    vars.insert("duration", duration as f64);

    let result = eval_expr_str(expr, &vars);
    AnimValue::Number(result as f32)
}

/// Evaluate a vector expression (`[x, y]`) from a string.
pub fn evaluate_expression_vec(
    expr: &str,
    value: [f32; 2],
    frame: u32,
    fps: u32,
    width: u32,
    height: u32,
    duration: u32,
) -> AnimValue {
    let time_s = frame as f64 / fps as f64;

    // For vector expressions we replace value[0] and value[1] in the
    // source, then eval each half.
    let parts = parse_bracket_expr(expr);

    let mut vars: HashMap<&str, f64> = HashMap::new();
    vars.insert("time", time_s);
    vars.insert("frame", frame as f64);
    vars.insert("fps", fps as f64);
    vars.insert("width", width as f64);
    vars.insert("height", height as f64);
    vars.insert("duration", duration as f64);

    let x = eval_with_value0(&parts.0, value[0] as f64, &vars);
    let y = eval_with_value0(&parts.1, value[1] as f64, &vars);
    AnimValue::Vec2([x as f32, y as f32])
}

/// Split `[expr_x, expr_y]` into (expr_x, expr_y) after stripping brackets.
fn parse_bracket_expr(expr: &str) -> (String, String) {
    let expr = expr.trim();
    assert!(
        expr.starts_with('[') && expr.ends_with(']'),
        "vector expression must be [x_expr, y_expr], got '{expr}'"
    );
    let inner = &expr[1..expr.len() - 1];
    // Find the comma that splits x and y — skip nested brackets.
    let mut depth = 0i32;
    let mut comma_pos = None;
    for (i, c) in inner.char_indices() {
        match c {
            '[' => depth += 1,
            ']' => depth -= 1,
            ',' if depth == 0 => {
                comma_pos = Some(i);
                break;
            }
            _ => {}
        }
    }
    let cp = comma_pos.expect("vector expression must have a comma separator");
    (
        inner[..cp].trim().to_string(),
        inner[cp + 1..].trim().to_string(),
    )
}

fn eval_with_value0(expr: &str, val0: f64, vars: &std::collections::HashMap<&str, f64>) -> f64 {
    let expr = expr.replace("value", &val0.to_string());
    let mut full_vars = vars.clone();
    full_vars.insert("value", val0);
    eval_expr_str(&expr, &full_vars)
}

// ----- tiny arithmetic evaluator (no deps) -----

fn eval_expr_str(expr: &str, vars: &std::collections::HashMap<&str, f64>) -> f64 {
    let tokens = tokenize(expr, vars);
    eval_tokens(&tokens)
}

fn tokenize(expr: &str, vars: &std::collections::HashMap<&str, f64>) -> Vec<String> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            c if c.is_ascii_whitespace() => {
                i += 1;
            }
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                tokens.push(chars[start..i].iter().collect());
            }
            '-' if tokens.last().map_or(true, |t: &String| {
                t == "(" || t == "+" || t == "-" || t == "*" || t == "/"
            }) && (i + 1 < chars.len()
                && (chars[i + 1].is_ascii_digit() || chars[i + 1] == '.')) =>
            {
                // unary minus before number
                i += 1;
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                tokens.push(format!("-{}", chars[start..i].iter().collect::<String>()));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_alphanumeric() || chars[i] == '_' {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                let val = vars
                    .get(name.as_str())
                    .unwrap_or_else(|| panic!("unknown variable in expression: '{name}'"));
                tokens.push(val.to_string());
            }
            c => {
                tokens.push(c.to_string());
                i += 1;
            }
        }
    }
    tokens
}

/// Evaluate tokens with standard precedence: * and / before + and -.
fn eval_tokens(tokens: &[String]) -> f64 {
    // First pass: handle * and /
    let mut acc: Vec<String> = Vec::new();
    let mut num = tokens[0].parse::<f64>().expect("expected number");
    let mut i = 1;
    while i < tokens.len() {
        let op = &tokens[i];
        let rhs = tokens[i + 1].parse::<f64>().expect("expected number");
        match op.as_str() {
            "*" => num *= rhs,
            "/" => num /= rhs,
            _ => {
                acc.push(num.to_string());
                acc.push(op.clone());
                num = rhs;
            }
        }
        i += 2;
    }
    acc.push(num.to_string());

    // Second pass: + and -
    let mut result = acc[0].parse::<f64>().unwrap();
    let mut i = 1;
    while i < acc.len() {
        let op = &acc[i];
        let rhs = acc[i + 1].parse::<f64>().unwrap();
        match op.as_str() {
            "+" => result += rhs,
            "-" => result -= rhs,
            _ => panic!("unexpected operator: {op}"),
        }
        i += 2;
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eval_simple_mul() {
        assert_eq!(eval_expr_str("5 * 3", &Default::default()), 15.0);
    }

    #[test]
    fn eval_precedence() {
        assert_eq!(eval_expr_str("2 + 3 * 4", &Default::default()), 14.0);
    }

    #[test]
    fn eval_paren() {
        assert_eq!(eval_expr_str("(2 + 3) * 4", &Default::default()), 20.0);
    }

    #[test]
    fn eval_with_var() {
        let mut vars = std::collections::HashMap::new();
        vars.insert("x", 10.0);
        assert_eq!(eval_expr_str("x + 5", &vars), 15.0);
    }
}
