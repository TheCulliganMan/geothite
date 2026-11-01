//! Selected-pack notebook mechanics, independent of the separate prose oracle.
use std::collections::BTreeSet;

use super::*;

#[test]
#[ignore = "requires selected Mankato pack and caller-named reached-save directory"]
fn selected_mankato_notebook_acknowledgment_repeat_and_continue() {
    let (root, r, mut c) = fixture("MnMankato", (26, 26), true);
    assert!(!event(&c, NOTE));
    let baseline = c.snapshot().expect("initial notebook approach");
    let badges = c.shell.shell.session().state().badges.clone();
    let initial_flags = c
        .shell
        .shell
        .session()
        .state()
        .flags
        .active_event_flags()
        .cloned()
        .collect::<BTreeSet<_>>();
    press(&mut c, GameButton::Up);
    press(&mut c, GameButton::A);
    let first = c
        .snapshot()
        .expect("first notebook page")
        .ui
        .text
        .and_then(|text| text.asm_text)
        .expect("actual first page");
    assert!(!first.is_empty());
    assert!(!event(&c, NOTE), "opening the clue is not acknowledgment");
    press(&mut c, GameButton::Down);
    at(&mut c, "MnMankato", (26, 26));
    assert!(!event(&c, NOTE), "dialogue must own movement input");
    press(&mut c, GameButton::A);
    let second = c
        .snapshot()
        .expect("second notebook page")
        .ui
        .text
        .and_then(|text| text.asm_text)
        .expect("actual second page");
    assert!(!second.is_empty());
    assert_ne!(first, second, "the two pages must actually advance");
    assert!(
        !event(&c, NOTE),
        "one acknowledgment cannot complete both pages"
    );
    finish(&mut c);
    assert!(event(&c, NOTE));
    for stage in ["mankato-selected-note-ack", "mankato-selected-note-repeat"] {
        let snapshot = c.snapshot().expect("completed notebook");
        assert_eq!(snapshot.party, baseline.party);
        assert_eq!(snapshot.bag, baseline.bag);
        assert_eq!(snapshot.trainer.money, baseline.trainer.money);
        assert_eq!(c.shell.shell.session().state().badges, badges);
        let mut flags = c
            .shell
            .shell
            .session()
            .state()
            .flags
            .active_event_flags()
            .cloned()
            .collect::<BTreeSet<_>>();
        assert!(flags.remove(NOTE));
        assert_eq!(
            flags, initial_flags,
            "no other story qualification is granted"
        );
        c = continued(c, &root, &r, stage);
        assert!(event(&c, NOTE), "Continue retains the acknowledged clue");
        press(&mut c, GameButton::Up);
        press(&mut c, GameButton::A);
        assert!(c.snapshot().expect("repeat clue").ui.text.is_some());
        finish(&mut c);
        assert!(event(&c, NOTE));
    }
}
