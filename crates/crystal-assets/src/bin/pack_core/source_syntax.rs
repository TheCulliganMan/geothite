use anyhow::Result;
pub(super) fn code_part(line: &str) -> &str {
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && quoted {
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
        }
        if ch == ';' && !quoted {
            return line[..index].trim();
        }
    }
    line.trim()
}

pub(super) fn source_arguments(source: &str) -> Result<Vec<String>> {
    if source.trim().is_empty() {
        return Ok(Vec::new());
    }
    let mut result = Vec::new();
    let mut start = 0;
    let mut quoted = false;
    let mut escaped = false;
    for (index, ch) in source.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' && quoted {
            escaped = true;
            continue;
        }
        if ch == '"' {
            quoted = !quoted;
        }
        if ch == ',' && !quoted {
            result.push(source[start..index].trim().to_string());
            start = index + 1;
        }
    }
    anyhow::ensure!(!quoted && !escaped, "unterminated radio text string");
    result.push(source[start..].trim().to_string());
    anyhow::ensure!(
        result.iter().all(|arg| !arg.is_empty()),
        "empty radio text operand"
    );
    Ok(result)
}

#[cfg(test)]
pub fn test_workspace() -> std::path::PathBuf {
    std::path::PathBuf::from(std::env::var_os("CRYSTAL_EXPORT_WORKSPACE")
        .expect("source tests require the initialized pinned external workspace"))
}

#[cfg(test)]
pub fn test_rom_offset(workspace: &std::path::Path, name: &str) -> usize {
    let symbols = std::fs::read_to_string(workspace.join("vendor/pokecrystal/pokecrystal.sym")).unwrap();
    let address = symbols.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let address = fields.next()?;
        (fields.next()? == name).then_some(address)
    }).unwrap_or_else(|| panic!("missing pinned ROM symbol {name}"));
    let (bank, address) = address.split_once(':').unwrap();
    let bank = usize::from_str_radix(bank, 16).unwrap();
    let address = usize::from_str_radix(address, 16).unwrap();
    bank * 0x4000 + address % 0x4000
}
