use std::collections::BTreeSet;

use crate::CrystalRuntime;

impl CrystalRuntime {
    /// Borrow the verified pack's immutable music membership table. Input and
    /// script transactions only query it; rebuilding it for every frame clones
    /// every track identifier and allocates another tree.
    pub(super) fn borrowed_music_ids(&self) -> &BTreeSet<String> {
        &self.static_catalog_cache().music_ids
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::ptr;

    use crystal_core::input::GameButton;

    use crate::RuntimeDividerSource;
    use crate::RuntimeGameShell;
    use crate::test_support::load_minimal_compiled_runtime;

    #[test]
    fn music_membership_is_shared_and_owned_results_cannot_change_it() {
        let (path, _, runtime) = load_minimal_compiled_runtime("borrowed-music-membership");
        let cloned = runtime.clone();
        assert_eq!(runtime.borrowed_music_ids(), &runtime.audio().music_ids());
        assert!(ptr::eq(
            runtime.borrowed_music_ids(),
            cloned.borrowed_music_ids()
        ));
        let mut owned = runtime.music_ids();
        owned.clear();
        assert!(owned.is_empty());
        assert!(!runtime.borrowed_music_ids().is_empty());
        fs::remove_dir_all(path).expect("remove membership fixture");
    }

    #[test]
    fn live_and_recorded_inputs_keep_identical_state_and_music() {
        let (path, root, runtime) = load_minimal_compiled_runtime("borrowed-music-inputs");
        let mut recorded = RuntimeGameShell::new_game(root, runtime, 0).expect("recorded shell");
        *recorded.session_mut().divider_mut_for_tests() = RuntimeDividerSource::replay([0; 1024]);
        let mut live = recorded.clone();
        live.set_runtime_journal_enabled(false);
        for input in [
            None,
            Some(GameButton::Right),
            None,
            Some(GameButton::Left),
            Some(GameButton::B),
            None,
        ]
        .into_iter()
        .cycle()
        .take(64)
        {
            recorded.tick_after_vblank(input).expect("recorded input");
            live.tick_after_vblank(input).expect("live input");
            assert_eq!(live.session(), recorded.session());
            assert_eq!(
                live.state_checksum().expect("live checksum"),
                recorded.state_checksum().expect("recorded checksum")
            );
        }
        fs::remove_dir_all(path).expect("remove input fixture");
    }
}
