//! Radio selection tables in their cartridge order, shared by live playback.
use crate::{GameDataSet, radio_text_memory::source_radio_map};
use anyhow::{Context, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioCatalog {
    pub oak_grass_routes: Vec<bool>,
    pub trainer_class_count: u8,
    pub place_count: u8,
    pub hidden_people: Vec<u8>,
    pub hidden_people_beat_e4: Vec<u8>,
    pub hidden_people_beat_kanto: Vec<u8>,
}

impl RadioCatalog {
    pub fn new(data: &GameDataSet) -> Result<Self> {
        let definitions = &data
            .global_scripts
            .as_ref()
            .context("radio global scripts are missing")?
            .definitions;
        let table = |label: &str| -> Result<&Vec<serde_json::Value>> {
            definitions
                .get(label)
                .and_then(|value| value.as_array())
                .with_context(|| format!("radio source table {label} is missing"))
        };
        let classes = table("RadioTrainerClasses")?;
        let trainer_class_count = u8::try_from(
            classes
                .len()
                .checked_sub(1)
                .context("empty radio trainer classes")?,
        )?;
        anyhow::ensure!(
            trainer_class_count > 1,
            "radio trainer classes have no selectable entries"
        );
        let hidden = |label: &str| -> Result<Vec<u8>> {
            let rows = table(label)?;
            let mut ids = Vec::new();
            for (index, row) in rows.iter().enumerate() {
                anyhow::ensure!(
                    row["command"] == "db",
                    "radio exclusion {label} has a non-db row"
                );
                let args = row["args"]
                    .as_array()
                    .context("radio exclusion arguments are missing")?;
                anyhow::ensure!(
                    args.len() == 1,
                    "radio exclusion row has multiple arguments"
                );
                let name = args[0]
                    .as_str()
                    .context("radio exclusion argument is not a constant")?;
                if name == "-1" {
                    anyhow::ensure!(
                        index + 1 == rows.len(),
                        "radio exclusion terminator is not last"
                    );
                    return Ok(ids);
                }
                let id = classes
                    .iter()
                    .position(|class| {
                        class["command"] == "trainerclass" && class["args"][0] == name
                    })
                    .with_context(|| format!("radio exclusion trainer {name} is missing"))?;
                ids.push(u8::try_from(id)?);
            }
            anyhow::bail!("radio exclusion {label} has no source terminator")
        };
        let route_count = table("OaksPKMNTalkRoutes")?.len();
        anyhow::ensure!((1..=32).contains(&route_count), "invalid Oak route count");
        let oak_grass_routes = (0..route_count)
            .map(|index| {
                let map = source_radio_map(data, "OaksPKMNTalkRoutes", u8::try_from(index)?)?;
                Ok(data
                    .wild_encounters
                    .get(map)
                    .and_then(|encounters| encounters.grass.as_ref())
                    .is_some())
            })
            .collect::<Result<Vec<_>>>()?;
        let place_count = u8::try_from(table("PnP_Places")?.len())?;
        anyhow::ensure!(place_count > 0, "radio place table is empty");
        for index in 0..place_count {
            source_radio_map(data, "PnP_Places", index)?;
        }
        Ok(Self {
            oak_grass_routes,
            trainer_class_count,
            place_count,
            hidden_people: hidden("PnP_HiddenPeople")?,
            hidden_people_beat_e4: hidden("PnP_HiddenPeople_BeatE4")?,
            hidden_people_beat_kanto: hidden("PnP_HiddenPeople_BeatKanto")?,
        })
    }
}
