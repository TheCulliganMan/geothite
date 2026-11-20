//! Tamarack's regional Pokédex report during the Ely induction.

use std::collections::VecDeque;

use crystal_core::models::PokedexState;

pub(super) struct Rating {
    pub(super) counts: String,
    pub(super) pages: VecDeque<String>,
    pub(super) fanfare: &'static str,
}

pub(super) fn rating<'a>(
    map: &str,
    dex: &PokedexState,
    members: impl Iterator<Item = &'a str>,
) -> Option<Rating> {
    if map != "MnLeagueHallOfFame" {
        return None;
    }
    let mut seen = 0;
    let mut caught = 0;
    let mut total = 0;
    for species in members {
        seen += usize::from(dex.has_seen(species));
        caught += usize::from(dex.has_caught(species));
        total += 1;
    }
    if total == 0 {
        return None;
    }
    let (message, fanfare) = match total - caught {
        0 => (
            "TAMARACK: Every\nlast one!\n\nI still have your\nfirst field notes.",
            "SFX_DEX_FANFARE_230_PLUS",
        ),
        1 => (
            "TAMARACK: Just one\nleft to catch!\n\nBring that one by\nthe field station.",
            "SFX_DEX_FANFARE_200_229",
        ),
        _ => (
            "TAMARACK: Which\none surprised you?\n\nBring your notes\nby the station.",
            "SFX_DEX_FANFARE_80_109",
        ),
    };
    Some(Rating {
        counts: format!("Seen: {seen}/{total}\nCaught: {caught}/{total}"),
        pages: message.split("\n\n").map(str::to_owned).collect(),
        fanfare,
    })
}

#[cfg(test)]
mod tests {
    use super::rating;
    use crystal_core::models::PokedexState;

    #[test]
    fn ely_recognizes_every_regional_partner_and_leaves_johto_alone() {
        let members: Vec<_> = (1..=225).map(|id| format!("REGIONAL{id}")).collect();
        let roster = || members.iter().map(String::as_str);
        let mut dex = PokedexState::default();
        dex.seen_species.extend(members.iter().cloned());
        dex.caught_species.extend(members[..224].iter().cloned());
        dex.caught_species.insert("DITTO".into());
        dex.seen_species.insert("DITTO".into());
        let almost = rating("MnLeagueHallOfFame", &dex, roster()).expect("Ely report");
        assert_eq!(almost.counts, "Seen: 225/225\nCaught: 224/225");
        assert_eq!(almost.pages[0], "TAMARACK: Just one\nleft to catch!");
        dex.caught_species.insert(members[224].clone());
        let complete = rating("MnLeagueHallOfFame", &dex, roster()).expect("Ely report");
        assert_eq!(complete.counts, "Seen: 225/225\nCaught: 225/225");
        assert_eq!(complete.pages[0], "TAMARACK: Every\nlast one!");
        assert_ne!(almost.fanfare, complete.fanfare);
        let start = rating("MnLeagueHallOfFame", &PokedexState::default(), roster())
            .expect("empty regional Dex still gets a report");
        assert_eq!(start.counts, "Seen: 0/225\nCaught: 0/225");
        for report in [almost, complete, start] {
            for page in std::iter::once(&report.counts).chain(&report.pages) {
                assert!(page.lines().count() <= 2);
                assert!(page.lines().all(|line| line.len() <= 18));
            }
        }
        assert!(rating("HallOfFame", &dex, roster()).is_none());
        assert!(rating("MnLeagueHallOfFame", &dex, std::iter::empty()).is_none());
    }
}
