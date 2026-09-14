use super::RadioTextError;
use crate::systems::script_text::ScriptTextBody;
use std::collections::BTreeMap;

/// Encode PlaceString input without expanding its controls into rendered glyphs.
/// Longest source charmap tokens (including contractions) occupy one byte.
pub fn encode_radio_string(text: &str) -> Result<Vec<u8>, RadioTextError> {
    let tokens = [
        ("<TRAINER>", 0x5d),
        ("<ROCKET>", 0x5e),
        ("<PKMN>", 0x4a),
        ("<POKE>", 0x24),
        ("<PC>", 0x5b),
        ("<TM>", 0x5c),
        ("<PK>", 0xe1),
        ("<MN>", 0xe2),
        ("<PO>", 0x70),
        ("<KE>", 0x71),
        ("<DOT>", 0xf2),
        ("<……>", 0x56),
        ("<BSP>", 0x1f),
        ("<WBR>", 0x25),
        ("<LINE>", 0x4f),
        ("<NEXT>", 0x4e),
        ("<LF>", 0x22),
        ("'d", 0xd0),
        ("'l", 0xd1),
        ("'m", 0xd2),
        ("'r", 0xd3),
        ("'s", 0xd4),
        ("'t", 0xd5),
        ("'v", 0xd6),
    ];
    let mut bytes = Vec::new();
    let mut offset = 0;
    while offset < text.len() {
        let rest = &text[offset..];
        if let Some((token, byte)) = tokens.iter().find(|(token, _)| rest.starts_with(token)) {
            bytes.push(*byte);
            offset += token.len();
            continue;
        }
        let ch = rest
            .chars()
            .next()
            .expect("nonempty remaining source string");
        let byte = match ch {
            'A'..='Z' => ch as u8 - b'A' + 0x80,
            'a'..='z' => ch as u8 - b'a' + 0xa0,
            '0'..='9' => ch as u8 - b'0' + 0xf6,
            ' ' => 0x7f,
            '@' => 0x50,
            '#' => 0x54,
            '(' => 0x9a,
            ')' => 0x9b,
            ':' => 0x9c,
            ';' => 0x9d,
            '[' => 0x9e,
            ']' => 0x9f,
            '\'' => 0xe0,
            '-' => 0xe3,
            '?' => 0xe6,
            '!' => 0xe7,
            '.' => 0xe8,
            '&' => 0xe9,
            'é' => 0xea,
            '×' => 0xf1,
            '/' => 0xf3,
            ',' => 0xf4,
            '♂' => 0xef,
            '♀' => 0xf5,
            '¥' => 0xf0,
            '…' => 0x75,
            '“' => 0x72,
            '”' => 0x73,
            _ => {
                return Err(RadioTextError::Encoding {
                    offset,
                    character: ch,
                });
            }
        };
        bytes.push(byte);
        offset += ch.len_utf8();
    }
    Ok(bytes)
}

/// Assemble exported source macros for RadioLinePrinter. RAM references remain
/// references supplied by the caller, rather than being flattened into prose.
pub fn encode_radio_text_body(
    body: &ScriptTextBody,
    ram_symbols: &BTreeMap<String, u16>,
) -> Result<Vec<u8>, RadioTextError> {
    let mut bytes = Vec::new();
    let mut terminated = false;
    for command in &body.commands {
        let error = |detail: &str| RadioTextError::Body {
            label: body.label.clone(),
            index: command.command_index,
            detail: detail.into(),
        };
        if terminated {
            return Err(error("command follows the source terminator"));
        }
        let (opcode, arguments) = match command.command.as_str() {
            "text_start" => (0, 0),
            "text" => (0, 1),
            "line" => (0x4f, 1),
            "text_ram" => (1, 1),
            "text_pause" => (0x0a, 0),
            "text_today" => (0x15, 0),
            "done" => (0x57, 0),
            "text_end" => (0x50, 0),
            _ => return Err(error("unsupported radio text macro")),
        };
        if command.args.len() != arguments {
            return Err(error("incorrect source operand count"));
        }
        bytes.push(opcode);
        match command.command.as_str() {
            "text" | "line" => {
                let text: String = serde_json::from_str(&command.args[0])
                    .map_err(|_| error("radio text operand must be a quoted source string"))?;
                bytes.extend(encode_radio_string(&text)?);
            }
            "text_ram" => {
                let symbol = &command.args[0];
                let address = ram_symbols
                    .get(symbol)
                    .ok_or_else(|| RadioTextError::MissingRamSymbol(symbol.clone()))?;
                bytes.extend(address.to_le_bytes());
            }
            "done" | "text_end" => terminated = true,
            _ => {}
        }
    }
    if !terminated {
        return Err(RadioTextError::Truncated);
    }
    Ok(bytes)
}

/// Reconstruct the source dex-entry byte layout consumed by PokedexShow2..8.
/// Catalog ` @ ` separators preserve source `next`; pages preserve `page`.
/// Never reflow prose: the radio reads exactly six source lines after category.
pub fn encode_radio_pokedex_entry(
    entry: &crate::models::pokedex::RuntimePokedexEntry,
) -> Result<Vec<u8>, RadioTextError> {
    let invalid = || RadioTextError::PokedexEntry(entry.species.clone());
    if entry.pages.len() != 2 {
        return Err(invalid());
    }
    let mut bytes = encode_radio_string(&entry.classification)?;
    if bytes.contains(&0x50) {
        return Err(invalid());
    }
    bytes.push(0x50);
    bytes.extend(entry.height_digits.to_le_bytes());
    bytes.extend(entry.weight_digits.to_le_bytes());
    for page in &entry.pages {
        let lines = page.split(" @ ").collect::<Vec<_>>();
        if lines.len() != 3 {
            return Err(invalid());
        }
        for (line_index, line) in lines.iter().enumerate() {
            let encoded = encode_radio_string(line)?;
            if encoded.len() > 18
                || encoded
                    .iter()
                    .any(|byte| matches!(byte, 0x50 | 0x4e | 0x5f))
            {
                return Err(invalid());
            }
            bytes.extend(encoded);
            // The source page macro emits @, not <DEXEND>.
            bytes.push(if line_index != 2 { 0x4e } else { 0x50 });
        }
    }
    Ok(bytes)
}
