//! Match retained scoped text identities against their original ASM operand.
use anyhow::Result;

use crate::GameDataSet;

pub(crate) fn validate_text_payload(
    data: &GameDataSet,
    path: &str,
    script: &str,
    index: usize,
    command: &str,
    saved: &[String],
) -> Result<()> {
    let body = data.saved_compiled_script_body(path, script)?;
    let raw = body
        .as_array()
        .and_then(|commands| commands.get(index))
        .and_then(|entry| entry.get("args"))
        .and_then(serde_json::Value::as_array)
        .filter(|args| args.len() == 1)
        .and_then(|args| args[0].as_str());
    let original = restore_original_operand(script, raw, saved)?;
    data.validate_saved_script_command_payload_reference(
        path,
        script,
        index,
        command,
        original.as_deref().unwrap_or(saved),
    )
}

fn restore_original_operand(
    script: &str,
    raw: Option<&str>,
    saved: &[String],
) -> Result<Option<Vec<String>>> {
    if let (Some(raw), [saved]) = (raw, saved)
        && raw.starts_with('.')
        && crate::resolve_local_script_label(script, raw)? == *saved
    {
        return Ok(Some(vec![raw.to_string()]));
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::restore_original_operand;

    #[test]
    fn resolves_only_the_exact_local_operand_in_its_original_parent_scope() {
        let saved = vec![".UseStrengthText@Script_UsedStrength".into()];
        assert_eq!(
            restore_original_operand("Script_UsedStrength", Some(".UseStrengthText"), &saved)
                .expect("native local text"),
            Some(vec![".UseStrengthText".into()])
        );
        for (script, raw) in [
            ("OtherScript", ".UseStrengthText"),
            ("Script_UsedStrength", ".OtherText"),
            ("Script_UsedStrength", "UseStrengthText"),
        ] {
            assert_eq!(
                restore_original_operand(script, Some(raw), &saved).expect("mismatched reference"),
                None
            );
        }
        assert!(
            restore_original_operand(
                "OtherScript",
                Some(".UseStrengthText@Script_UsedStrength"),
                &saved
            )
            .is_err()
        );
    }
}
