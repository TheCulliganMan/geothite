//! Relative progress across the observed, wrapping Pack tab strip.
use serde_json::Value;
pub(crate) fn goal(v: &Value, wanted: &str) -> Option<(usize, &'static str)> {
    let menu = v.pointer("/observe/menus")?.as_array()?.last()?;
    let tabs = menu["pack_pockets"].as_array()?;
    let current = menu["pack_pocket"].as_str()?;
    let index = |name: &str| tabs.iter().position(|s| s.as_str().is_some_and(|s| s.eq_ignore_ascii_case(name)));
    let a = index(current)?;
    let b = index(wanted)?;
    let east = (b + tabs.len() - a) % tabs.len();
    let west = (a + tabs.len() - b) % tabs.len();
    Some((east.min(west), if east == 0 { "aligned" } else if east <= west { "east" } else { "west" }))
}
pub(crate) fn cue(v: &Value, wanted: &str) -> Option<String> {
    goal(v, wanted).map(|(_, direction)| format!("objective:pocket:{wanted}:{direction}"))
}
#[cfg(test)] mod tests {
    use super::*;
    use serde_json::json;
    #[test] fn uses_observed_order_and_wraps_without_inventing_missing_tabs() {
        let v=json!({"observe":{"menus":[{"pack_pockets":["Items","Balls","Key","TM/HM","Custom"],"pack_pocket":"Items"}]}});
        assert_eq!(goal(&v,"Balls"),Some((1,"east")));
        assert_eq!(goal(&v,"TM/HM"),Some((2,"west")));
        assert_eq!(goal(&v,"Absent"),None);
        assert_eq!(goal(&json!({}),"Balls"),None);
    }
}
