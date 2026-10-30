//! Show source save text before allowing its final yes/no answer.
use anyhow::{Context, Result, ensure};
use crystal_runtime::RuntimeShellSnapshot;

use super::{BevyRuntimeShell, SpecialBoundaryDisplay, visible_bill_pc_save_pages};

pub(super) fn begin(
    shell: &mut BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    source_label: &str,
    display_label: &str,
) -> Result<()> {
    let mut pages = visible_bill_pc_save_pages(shell, snapshot, source_label)?;
    pages
        .pop()
        .context("save question has no final source page")?;
    if !pages.is_empty() {
        ensure!(
            shell.special_boundary.is_none(),
            "another notice owns the save prompt"
        );
        shell.special_boundary = Some(SpecialBoundaryDisplay {
            label: display_label.into(),
            details: pages,
        });
        shell.field_text_reveal = None;
    }
    Ok(())
}

pub(super) fn final_page(
    shell: &BevyRuntimeShell,
    snapshot: &RuntimeShellSnapshot,
    source_label: &str,
) -> Result<String> {
    visible_bill_pc_save_pages(shell, snapshot, source_label)?
        .pop()
        .context("save question has no final source page")
}
