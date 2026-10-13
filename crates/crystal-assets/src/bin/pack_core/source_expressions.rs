use anyhow::{Context, Result, ensure};
use std::collections::BTreeMap;

pub(super) fn evaluate(input: &str, constants: &BTreeMap<String, i64>) -> Result<i64> {
    let mut input = input.trim();
    while input.starts_with('(') && input.ends_with(')') {
        let mut depth = 0;
        let mut outer = true;
        for (index, byte) in input.bytes().enumerate() {
            if byte == b'(' {
                depth += 1;
            }
            if byte == b')' {
                depth -= 1;
            }
            if depth == 0 && index + 1 < input.len() {
                outer = false;
                break;
            }
        }
        if !outer {
            break;
        }
        input = input[1..input.len() - 1].trim();
    }
    for operators in [
        &["|"][..],
        &["^"][..],
        &["&"][..],
        &["<<", ">>"][..],
        &["+", "-"][..],
        &["*", "/", "%"][..],
    ] {
        let mut depth = 0;
        let mut split = None;
        for (index, byte) in input.bytes().enumerate() {
            if byte == b'(' {
                depth += 1;
                continue;
            }
            if byte == b')' {
                depth -= 1;
                continue;
            }
            if depth != 0 || index == 0 {
                continue;
            }
            for operator in operators {
                if input[index..].starts_with(operator) {
                    split = Some((index, *operator));
                }
            }
        }
        if let Some((index, operator)) = split {
            let left = evaluate(&input[..index], constants)?;
            let right = evaluate(&input[index + operator.len()..], constants)?;
            return match operator {
                "|" => Ok(left | right),
                "^" => Ok(left ^ right),
                "&" => Ok(left & right),
                "<<" => left
                    .checked_shl(u32::try_from(right)?)
                    .context("source shift overflow"),
                ">>" => left
                    .checked_shr(u32::try_from(right)?)
                    .context("source shift overflow"),
                "+" => left.checked_add(right).context("source addition overflow"),
                "-" => left
                    .checked_sub(right)
                    .context("source subtraction overflow"),
                "*" => left
                    .checked_mul(right)
                    .context("source multiplication overflow"),
                "/" => left.checked_div(right).context("source division overflow"),
                "%" => left.checked_rem(right).context("source remainder overflow"),
                _ => unreachable!(),
            };
        }
    }
    if let Some(value) = input.strip_prefix('-') {
        return evaluate(value, constants)?
            .checked_neg()
            .context("source negation overflow");
    }
    if let Some(value) = input.strip_prefix('$') {
        return Ok(i64::from_str_radix(value, 16)?);
    }
    if let Some(value) = input.strip_prefix('%') {
        return Ok(i64::from_str_radix(value, 2)?);
    }
    input
        .parse()
        .ok()
        .or_else(|| constants.get(input).copied())
        .with_context(|| format!("unresolved numeric source expression {input}"))
}

pub(super) fn normalize(
    source: &str,
    scripts: &mut BTreeMap<String, serde_json::Value>,
    global: &BTreeMap<String, i64>,
) -> Result<()> {
    let mut constants = global.clone();
    let mut in_macro = false;
    for line in source.lines().map(super::radio_text_export::code_part) {
        if line.starts_with("MACRO ") {
            in_macro = true;
        }
        if line == "ENDM" {
            in_macro = false;
            continue;
        }
        if in_macro {
            continue;
        }
        if let Some(definition) = line.strip_prefix("DEF ") {
            if let Some((name, value)) = definition
                .split_once(" EQU ")
                .or_else(|| definition.split_once(" = "))
            {
                let name = name.trim();
                if name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                {
                    if let Ok(value) = evaluate(value, &constants) {
                        constants.insert(name.to_owned(), value);
                    }
                }
            }
        }
    }
    for commands in scripts.values_mut() {
        for command in commands.as_array_mut().context("script commands")? {
            for argument in command["args"].as_array_mut().context("script arguments")? {
                let mut token = argument
                    .as_str()
                    .context("source argument token")?
                    .to_owned();
                while let Some(start) = token.find("{d:") {
                    let end = start
                        + token[start..]
                            .find('}')
                            .context("unterminated source interpolation")?;
                    let value = evaluate(&token[start + 3..end], &constants)?;
                    token.replace_range(start..=end, &value.to_string());
                }
                if !token.starts_with('"')
                    && token.contains(char::is_whitespace)
                    && token.chars().any(|ch| "()+-*/|&<>".contains(ch))
                {
                    token = evaluate(&token, &constants)?.to_string();
                }
                ensure!(!token.contains("{d:"), "unexpanded source interpolation");
                *argument = serde_json::Value::String(token);
            }
        }
    }
    Ok(())
}
