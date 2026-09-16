use crate::project::{
    AnimProp, AnimValue, Easing, Expression, KeyTime, Keyframe, Layer, Transform,
};

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
/// `Step` returns 0.0 — the interpolation caller treats it as a hold.
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

// ---------------------------------------------------------------------------
// Expression evaluator (v0.2 — pure-Rust arithmetic, no Python dependency)
// ---------------------------------------------------------------------------

/// Build the variable map from project context.
fn build_vars(frame: u32, fps: u32, width: u32, height: u32, duration: u32) -> Vec<(String, f64)> {
    vec![
        ("frame".into(), frame as f64),
        ("time".into(), frame as f64 / fps as f64),
        ("fps".into(), fps as f64),
        ("width".into(), width as f64),
        ("height".into(), height as f64),
        ("duration".into(), duration as f64),
    ]
}

/// Evaluate a scalar expression (`"frame * 2.0"`). Panics on syntax or type error (v0.2).
pub fn evaluate_expression(
    expr: &str,
    value: AnimValue,
    frame: u32,
    fps: u32,
    width: u32,
    height: u32,
    duration: u32,
) -> AnimValue {
    let val_num = match value {
        AnimValue::Number(n) => n as f64,
        _ => panic!("expression got non-scalar value for a scalar property"),
    };
    let mut vars = build_vars(frame, fps, width, height, duration);
    vars.push(("value".into(), val_num));

    let result = eval_arithmetic(expr, &vars);
    AnimValue::Number(result as f32)
}

/// Evaluate a vector expression (`"[expr_x, expr_y]"`). Panics on syntax or type error.
pub fn evaluate_expression_vec(
    expr: &str,
    value: [f32; 2],
    frame: u32,
    fps: u32,
    width: u32,
    height: u32,
    duration: u32,
) -> AnimValue {
    let expr = expr.trim();
    assert!(
        expr.starts_with('[') && expr.ends_with(']'),
        "expression syntax error: vector expression must be [x_expr, y_expr], got '{expr}'"
    );
    let inner = &expr[1..expr.len() - 1];

    // Split at top-level comma (skip nested brackets)
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
    let cp =
        comma_pos.expect("expression syntax error: vector expression must have a comma separator");
    let ex = inner[..cp].trim().to_string();
    let ey = inner[cp + 1..].trim().to_string();

    // value[0]/value[1] -> components; remaining bare `value` -> that side's component
    let ex = ex
        .replace("value[0]", &value[0].to_string())
        .replace("value[1]", &value[1].to_string())
        .replace("value", &value[0].to_string());
    let ey = ey
        .replace("value[0]", &value[0].to_string())
        .replace("value[1]", &value[1].to_string())
        .replace("value", &value[1].to_string());

    let mut vars = build_vars(frame, fps, width, height, duration);
    let x = eval_arithmetic(&ex, &vars);
    vars.push(("value".into(), value[0] as f64));
    let y = eval_arithmetic(&ey, &vars);

    AnimValue::Vec2([x as f32, y as f32])
}

// ---------------------------------------------------------------------------
// Tiny arithmetic evaluator (no deps)
// ---------------------------------------------------------------------------

enum Token {
    Num(f64),
    Op(char),
}

fn tokenize(expr: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        match chars[i] {
            c if c.is_ascii_whitespace() => i += 1,
            '(' => {
                tokens.push(Token::Op('('));
                i += 1;
            }
            ')' => {
                tokens.push(Token::Op(')'));
                i += 1;
            }
            '+' | '*' | '/' => {
                tokens.push(Token::Op(chars[i]));
                i += 1;
            }
            '-' => {
                // unary minus if at start or after an op/open-paren
                let is_unary = matches!(
                    tokens.last(),
                    None | Some(
                        Token::Op('(')
                            | Token::Op('+')
                            | Token::Op('-')
                            | Token::Op('*')
                            | Token::Op('/')
                    )
                );
                if is_unary {
                    i += 1;
                    // consume number after unary minus
                    let mut num_str = String::from("-");
                    while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                        num_str.push(chars[i]);
                        i += 1;
                    }
                    tokens.push(Token::Num(
                        num_str
                            .parse::<f64>()
                            .expect("expression syntax error: bad number"),
                    ));
                } else {
                    tokens.push(Token::Op('-'));
                    i += 1;
                }
            }
            '0'..='9' | '.' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_digit() || chars[i] == '.') {
                    i += 1;
                }
                let s: String = chars[start..i].iter().collect();
                tokens.push(Token::Num(
                    s.parse::<f64>()
                        .expect("expression syntax error: bad number"),
                ));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let start = i;
                while i < chars.len() && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                    i += 1;
                }
                let name: String = chars[start..i].iter().collect();
                // Variable lookup done at evaluate time — push a sentinel
                // We'll resolve it inline in the expression string replacement.
                panic!("expression syntax error: unknown variable '{name}' (variables must be substituted before evaluation)");
            }
            c => panic!("expression must be a number expression, unexpected character '{c}'"),
        }
    }
    tokens
}

/// Evaluate arithmetic with `+`/`-` and `*`/`/` precedence and parenthesised sub-expressions.
fn eval_arithmetic(expr: &str, vars: &[(String, f64)]) -> f64 {
    // Substitute variable names with their numeric values first
    let mut s = expr.trim().to_string();
    for (name, val) in vars {
        // Replace whole-word occurrences only (not substrings of other names)
        let marker = format!("__{name}__");
        s = s.replace(name, &marker);
    }
    for (name, val) in vars {
        let marker = format!("__{name}__");
        s = s.replace(&marker, &val.to_string());
    }

    let tokens = tokenize(&s);
    eval_tokens(&tokens).expect("expression syntax error: malformed expression")
}

fn eval_tokens(tokens: &[Token]) -> Result<f64, String> {
    if tokens.is_empty() {
        return Err("expression syntax error: empty expression".into());
    }

    // Flatten into values + ops, handling parens recursively
    let mut values: Vec<f64> = Vec::new();
    let mut ops: Vec<char> = Vec::new();
    let mut i = 0;
    while i < tokens.len() {
        match &tokens[i] {
            Token::Num(n) => values.push(*n),
            Token::Op('(') => {
                // Find matching ')', recurse
                let mut depth = 1i32;
                let start = i + 1;
                let mut j = start;
                while j < tokens.len() && depth > 0 {
                    match &tokens[j] {
                        Token::Op('(') => depth += 1,
                        Token::Op(')') => depth -= 1,
                        _ => {}
                    }
                    if depth > 0 {
                        j += 1;
                    }
                }
                if depth != 0 {
                    return Err("expression syntax error: unmatched parentheses".into());
                }
                let sub = &tokens[start..j];
                let val = eval_tokens(sub)?;
                values.push(val);
                i = j + 1; // skip past ')'
                continue;
            }
            Token::Op(op) => {
                // Apply higher-precedence ops on the stack first
                while let Some(&top) = ops.last() {
                    if (top == '*' || top == '/') && (*op == '+' || *op == '-') {
                        apply_op(&mut values, top)?;
                    } else {
                        break;
                    }
                }
                ops.push(*op);
            }
        }
        i += 1;
    }
    // Apply remaining ops
    while let Some(op) = ops.pop() {
        apply_op(&mut values, op)?;
    }
    values
        .pop()
        .ok_or_else(|| "expression syntax error: no result".to_string())
}

fn apply_op(values: &mut Vec<f64>, op: char) -> Result<(), String> {
    let b = values
        .pop()
        .ok_or("expression syntax error: missing operand")?;
    let a = values
        .pop()
        .ok_or("expression syntax error: missing operand")?;
    match op {
        '+' => values.push(a + b),
        '-' => values.push(a - b),
        '*' => values.push(a * b),
        '/' => values.push(a / b),
        _ => return Err(format!("expression syntax error: unknown operator '{op}'")),
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// Resolve transform with expression override
// ---------------------------------------------------------------------------

/// Resolve a full layer transform at a given frame, including expression overrides.
pub fn resolve_transform(
    layer: &Layer,
    frame: u32,
    fps: u32,
    width: u32,
    height: u32,
    duration: u32,
) -> Transform {
    let t = layer.transform;

    // 1. Resolve from keyframes (or static default)
    let position = resolve_vec2(&layer.keyframes, AnimProp::Position, frame, fps, t.position);
    let scale = resolve_vec2(&layer.keyframes, AnimProp::Scale, frame, fps, t.scale);
    let opacity = resolve_f32(&layer.keyframes, AnimProp::Opacity, frame, fps, t.opacity);
    let rotation = resolve_f32(&layer.keyframes, AnimProp::Rotation, frame, fps, t.rotation);

    // 2. Expression override: if an expression targets this property, it wins
    let position = apply_expr_vec(
        &layer.expressions,
        AnimProp::Position,
        position,
        frame,
        fps,
        width,
        height,
        duration,
    );
    let scale = apply_expr_vec(
        &layer.expressions,
        AnimProp::Scale,
        scale,
        frame,
        fps,
        width,
        height,
        duration,
    );
    let opacity = apply_expr_scalar(
        &layer.expressions,
        AnimProp::Opacity,
        opacity,
        frame,
        fps,
        width,
        height,
        duration,
    );
    let rotation = apply_expr_scalar(
        &layer.expressions,
        AnimProp::Rotation,
        rotation,
        frame,
        fps,
        width,
        height,
        duration,
    );

    Transform {
        position,
        scale,
        opacity,
        rotation,
    }
}

fn apply_expr_scalar(
    exprs: &[Expression],
    prop: AnimProp,
    current: f32,
    frame: u32,
    fps: u32,
    width: u32,
    height: u32,
    duration: u32,
) -> f32 {
    match exprs.iter().find(|e| e.property == prop) {
        Some(e) => match evaluate_expression(
            &e.expr,
            AnimValue::Number(current),
            frame,
            fps,
            width,
            height,
            duration,
        ) {
            AnimValue::Number(v) => v,
            _ => unreachable!(),
        },
        None => current,
    }
}

fn apply_expr_vec(
    exprs: &[Expression],
    prop: AnimProp,
    current: [f32; 2],
    frame: u32,
    fps: u32,
    width: u32,
    height: u32,
    duration: u32,
) -> [f32; 2] {
    match exprs.iter().find(|e| e.property == prop) {
        Some(e) => {
            match evaluate_expression_vec(&e.expr, current, frame, fps, width, height, duration) {
                AnimValue::Vec2(v) => v,
                _ => unreachable!(),
            }
        }
        None => current,
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
