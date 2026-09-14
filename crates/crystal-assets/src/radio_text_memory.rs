//! Catalog-backed RAM strings consumed by Crystal's radio text interpreter.
use std::collections::BTreeMap;

use crate::GameDataSet;
use anyhow::{Context, Result};
use crystal_core::systems::radio_program::{RadioPokedexEntryCursor, RadioProgramTextSource};
use crystal_core::systems::radio_text::{
    RadioTextEnvironment, RadioTextError, encode_radio_pokedex_entry, encode_radio_string,
    encode_radio_text_body,
};

// Source wram.asm addresses, also checked by the ROM radio trace fixtures.
pub const RADIO_MON_NAME: u16 = 0xd050;
pub const RADIO_STRING_1: u16 = 0xd073;
pub const RADIO_STRING_2: u16 = 0xd086;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadioTextMemory {
    weekday: u8,
    ram: BTreeMap<u16, Vec<u8>>,
    current_species: Option<u8>,
    dex_entry: Option<Vec<u8>>,
    dex_cursor: RadioPokedexEntryCursor,
}

impl RadioTextMemory {
    pub fn new(weekday: u8) -> Result<Self> {
        anyhow::ensure!(weekday < 7, "invalid radio weekday {weekday}");
        Ok(Self {
            weekday,
            ram: BTreeMap::new(),
            current_species: None,
            dex_entry: None,
            dex_cursor: RadioPokedexEntryCursor::default(),
        })
    }

    pub fn set_weekday(&mut self, weekday: u8) -> Result<()> {
        anyhow::ensure!(weekday < 7, "invalid radio weekday {weekday}");
        self.weekday = weekday;
        Ok(())
    }

    pub fn text(&mut self, data: &GameDataSet, source: &RadioProgramTextSource) -> Result<Vec<u8>> {
        match source {
            RadioProgramTextSource::Label(label) => self.label_text(data, label),
            RadioProgramTextSource::PokedexCategory => self.pokedex_text(true),
            RadioProgramTextSource::PokedexEntryLine => self.pokedex_text(false),
            RadioProgramTextSource::RetainedBuffer => {
                anyhow::bail!("retained radio text belongs to RadioPlayback")
            }
        }
    }

    pub fn label_text(&self, data: &GameDataSet, label: &str) -> Result<Vec<u8>> {
        let globals = data
            .global_scripts
            .as_ref()
            .context("radio global scripts are missing")?;
        let body = globals
            .script_text_bodies
            .get(label)
            .with_context(|| format!("radio text body {label} is missing"))?;
        Ok(encode_radio_text_body(
            body,
            &BTreeMap::from([
                ("wMonOrItemNameBuffer".into(), RADIO_MON_NAME),
                ("wStringBuffer1".into(), RADIO_STRING_1),
                ("wStringBuffer2".into(), RADIO_STRING_2),
            ]),
        )?)
    }

    /// GetPokemonName copies ten bytes and appends @, including for index zero.
    pub fn pokemon_name(data: &GameDataSet, species: u8) -> Result<Vec<u8>> {
        let row = data
            .global_scripts
            .as_ref()
            .context("radio global scripts are missing")?
            .definitions
            .get("PokemonNames")
            .and_then(|table| table.as_array())
            .context("source PokemonNames table is missing")?
            .get(usize::from(species.wrapping_sub(1)))
            .context("source PokemonNames slot is missing")?;
        anyhow::ensure!(
            row["command"] == "dname",
            "invalid source PokemonNames command"
        );
        let args = row["args"]
            .as_array()
            .context("invalid source PokemonNames arguments")?;
        anyhow::ensure!(
            args.len() == 1,
            "invalid source PokemonNames argument count"
        );
        let name: String = serde_json::from_str(
            args[0]
                .as_str()
                .context("invalid source PokemonNames string")?,
        )?;
        let mut bytes = encode_radio_string(&name)?;
        anyhow::ensure!(
            bytes.len() <= 10 && !bytes.contains(&0x50),
            "invalid source PokemonNames width or terminator"
        );
        bytes.resize(11, 0x50);
        Ok(bytes)
    }

    pub fn format_species(&mut self, data: &GameDataSet, species: u8) -> Result<()> {
        let name = Self::pokemon_name(data, species)?;
        self.current_species = Some(species);
        self.ram.insert(RADIO_STRING_1, name);
        Ok(())
    }

    pub fn format_current_species(&mut self, data: &GameDataSet) -> Result<()> {
        self.format_species(
            data,
            self.current_species
                .context("radio current species is unset")?,
        )
    }

    pub fn select_pokedex_species(&mut self, data: &GameDataSet, species: u8) -> Result<()> {
        let pokemon = data
            .pokemon
            .values()
            .find(|pokemon| pokemon.int_id == u16::from(species))
            .context("radio Pokédex species is missing")?;
        let entry = data
            .pokedex_entries
            .get(&pokemon.id)
            .context("radio Pokédex entry is missing")?;
        let bytes = encode_radio_pokedex_entry(entry)?;
        self.format_species(data, species)?;
        self.dex_entry = Some(bytes);
        self.dex_cursor = RadioPokedexEntryCursor::default();
        Ok(())
    }

    pub fn pokedex_text(&mut self, category: bool) -> Result<Vec<u8>> {
        Ok(self.dex_cursor.copy_text(
            self.dex_entry
                .as_deref()
                .context("radio Pokédex entry is unset")?,
            category,
        )?)
    }

    pub fn format_lucky_number(&mut self, number: u16) -> Result<()> {
        self.ram.insert(
            RADIO_STRING_1,
            encode_radio_string(&format!("{number:05}@"))?,
        );
        Ok(())
    }

    pub fn format_source_place(&mut self, data: &GameDataSet, index: u8) -> Result<()> {
        let map = source_radio_map(data, "PnP_Places", index)?;
        self.format_landmark(data, source_map_landmark(data, map)?)
    }

    pub fn format_oak_encounter(
        &mut self,
        data: &GameDataSet,
        route: u8,
        time: u8,
        slot: u8,
    ) -> Result<()> {
        let map = source_radio_map(data, "OaksPKMNTalkRoutes", route)?;
        let grass = data
            .wild_encounters
            .get(map)
            .and_then(|data| data.grass.as_ref())
            .context("Oak source grass table is missing")?;
        let time = match time {
            0 => crystal_core::world::encounters::TimeOfDay::Morning,
            1 => crystal_core::world::encounters::TimeOfDay::Day,
            2 => crystal_core::world::encounters::TimeOfDay::Night,
            _ => anyhow::bail!("Oak cannot select the darkness encounter period"),
        };
        anyhow::ensure!(
            (2..5).contains(&slot),
            "Oak source encounter slot is outside 2..5"
        );
        let encounter = grass
            .slots(time)
            .get(usize::from(slot))
            .context("Oak source encounter slot is missing")?;
        let species = data
            .pokemon
            .get(&encounter.species)
            .context("Oak source encounter species is missing")?;
        let landmark = source_map_landmark(data, map)?;
        let mut next = self.clone();
        next.format_species(data, u8::try_from(species.int_id)?)?;
        next.preserve_oak_species_name()?;
        next.format_landmark(data, landmark)?;
        *self = next;
        Ok(())
    }

    pub fn format_landmark(&mut self, data: &GameDataSet, landmark: u16) -> Result<()> {
        let definitions = &data
            .global_scripts
            .as_ref()
            .context("radio global scripts are missing")?
            .definitions;
        let row = definitions
            .get("Landmarks")
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.get(usize::from(landmark)))
            .context("radio source landmark is missing")?;
        let label = row["args"][2]
            .as_str()
            .context("radio source landmark has no name reference")?;
        let quoted = definitions
            .get(label)
            .and_then(|v| v.get(0))
            .and_then(|v| v.get("args"))
            .and_then(|v| v.get(0))
            .and_then(|v| v.as_str())
            .context("radio source landmark name is missing")?;
        let name: String = serde_json::from_str(quoted)?;
        let bytes = encode_radio_string(&name)?;
        anyhow::ensure!(
            bytes.last() == Some(&0x50),
            "radio source landmark is unterminated"
        );
        self.ram.insert(RADIO_STRING_1, bytes);
        Ok(())
    }

    /// Oak preserves the Pokémon's eleven bytes before GetLandmarkName.
    pub fn preserve_oak_species_name(&mut self) -> Result<()> {
        let name = self
            .ram
            .get(&RADIO_STRING_1)
            .context("Oak Pokémon name is unset")?
            .clone();
        anyhow::ensure!(name.len() == 11, "Oak Pokémon name is not eleven bytes");
        self.ram.insert(RADIO_MON_NAME, name);
        Ok(())
    }

    pub fn format_source_trainer(
        &mut self,
        data: &GameDataSet,
        class_id: u8,
        trainer_id: u8,
    ) -> Result<()> {
        let definitions = &data
            .global_scripts
            .as_ref()
            .context("radio global scripts are missing")?
            .definitions;
        let class = definitions
            .get("RadioTrainerClasses")
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.get(usize::from(class_id)))
            .and_then(|row| row["args"][0].as_str())
            .context("radio source trainer class is missing")?;
        let index = trainer_id
            .checked_sub(1)
            .context("radio source trainer index is zero")?;
        let table = format!("RadioTrainerIds_{class}");
        let key = definitions
            .get(&table)
            .and_then(|v| v.as_array())
            .and_then(|rows| rows.get(usize::from(index)))
            .and_then(|row| row["args"][0].as_str())
            .context("radio source trainer slot is missing")?;
        let trainer = data
            .trainers
            .trainers
            .get(key)
            .context("radio source trainer is missing from the catalog")?;
        anyhow::ensure!(
            trainer.trainer_class == class,
            "radio trainer slot belongs to the wrong source class"
        );
        self.format_trainer(data, key)
    }

    pub fn format_trainer(&mut self, data: &GameDataSet, trainer_key: &str) -> Result<()> {
        let trainer = data
            .trainers
            .trainers
            .get(trainer_key)
            .context("radio trainer is missing")?;
        let class = data
            .trainer_class_names
            .get(&trainer.trainer_class)
            .context("radio trainer class name is missing")?;
        let class_name = terminated_string(class)?;
        let trainer_name = terminated_string(&trainer.name)?;
        self.ram.insert(RADIO_STRING_2, class_name);
        self.ram.insert(RADIO_STRING_1, trainer_name);
        Ok(())
    }

    pub fn format_buena_password(&mut self, data: &GameDataSet, password: u8) -> Result<()> {
        let category_name = data
            .buena_password_categories
            .order
            .get(usize::from(password >> 4))
            .context("radio Buena category is missing")?;
        let category = data
            .buena_password_categories
            .categories
            .get(category_name)
            .context("radio Buena category data is missing")?;
        let value = category
            .options
            .get(usize::from(password & 15))
            .context("radio Buena word is missing")?;
        let bytes = match category.category_type.as_str() {
            "BUENA_MON" => {
                let species = data
                    .pokemon
                    .get(value)
                    .context("radio Buena Pokémon is missing")?;
                Self::pokemon_name(data, u8::try_from(species.int_id)?)?
            }
            "BUENA_ITEM" => terminated_string(
                &data
                    .items
                    .get(value)
                    .context("radio Buena item is missing")?
                    .name,
            )?,
            "BUENA_MOVE" => {
                let mov = data
                    .moves
                    .get(value)
                    .context("radio Buena move is missing")?;
                let index = mov
                    .source_index
                    .checked_sub(1)
                    .context("radio Buena move index is zero")?;
                terminated_string(
                    data.move_names
                        .get(usize::from(index))
                        .context("radio Buena move name is missing")?,
                )?
            }
            "BUENA_STRING" => terminated_string(value)?,
            other => anyhow::bail!("invalid radio Buena category type {other}"),
        };
        self.ram.insert(RADIO_STRING_1, bytes);
        Ok(())
    }
}

pub(crate) fn source_radio_map<'a>(data: &'a GameDataSet, table: &str, index: u8) -> Result<&'a str> {
    let definitions = &data
        .global_scripts
        .as_ref()
        .context("radio global scripts are missing")?
        .definitions;
    let row = definitions
        .get(table)
        .and_then(|v| v.as_array())
        .and_then(|rows| rows.get(usize::from(index)))
        .with_context(|| format!("source radio map table {table} index {index} is missing"))?;
    anyhow::ensure!(
        row["command"] == "map_id",
        "source radio map row is not map_id"
    );
    let constant = row["args"][0]
        .as_str()
        .context("source radio map constant is missing")?;
    data.runtime_map_metadata
        .iter()
        .find(|(_, metadata)| metadata.constant == constant)
        .map(|(_, metadata)| metadata.name.as_str())
        .context("source radio map metadata is missing")
}

fn source_map_landmark(data: &GameDataSet, map: &str) -> Result<u16> {
    let constant = data
        .pokegear_landmarks
        .map_to_landmark
        .get(map)
        .context("source radio map landmark is missing")?;
    data.pokegear_landmarks
        .landmarks
        .iter()
        .find(|landmark| landmark.constant == *constant)
        .map(|landmark| landmark.id)
        .context("source radio landmark ID is missing")
}

fn terminated_string(text: &str) -> Result<Vec<u8>> {
    let mut bytes = encode_radio_string(text)?;
    if bytes.last() != Some(&0x50) {
        bytes.push(0x50);
    }
    anyhow::ensure!(
        !bytes[..bytes.len() - 1].contains(&0x50),
        "embedded radio string terminator"
    );
    Ok(bytes)
}

impl RadioTextEnvironment for RadioTextMemory {
    fn ram_text(&self, address: u16) -> Result<Vec<u8>, RadioTextError> {
        self.ram
            .get(&address)
            .cloned()
            .ok_or(RadioTextError::Ram(address))
    }
    fn weekday(&self) -> u8 {
        self.weekday
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hex(text: &str) -> Vec<u8> {
        text.as_bytes()
            .chunks_exact(2)
            .map(|bytes| u8::from_str_radix(std::str::from_utf8(bytes).unwrap(), 16).unwrap())
            .collect()
    }

    #[test]
    fn radio_memory_formats_compiled_catalogs_like_source_ram() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../..")
            .canonicalize()
            .unwrap();
        let pack = crate::read_verified_compiled_game_pack(
            root.join("content-packs/core-modular.crystalpack"),
        )
        .unwrap();
        let data = pack.data();
        let source_names =
            include_bytes!("../../../../tools/asm-oracle/fixtures/pokemon-names.bin");
        for index in 0..256 {
            let name = RadioTextMemory::pokemon_name(data, (index as u8).wrapping_add(1)).unwrap();
            assert_eq!(&name[..10], &source_names[index * 10..index * 10 + 10]);
            assert_eq!(name[10], 0x50);
        }
        let mut memory = RadioTextMemory::new(0).unwrap();
        assert!(memory.ram_text(RADIO_STRING_1).is_err());
        memory.select_pokedex_species(data, 155).unwrap();
        let trace: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tools/asm-oracle/fixtures/radio-program-pokedex.json"
        ))
        .unwrap();
        let captured = hex(trace["prints"][0]["ram_text"]["53363"].as_str().unwrap());
        assert_eq!(memory.ram_text(RADIO_STRING_1).unwrap(), captured[..11]);
        for (index, category) in [true, false, false, false, false, false, false]
            .into_iter()
            .enumerate()
        {
            let text = memory.pokedex_text(category).unwrap();
            let expected = hex(trace["prints"][index + 1]["text_bytes"].as_str().unwrap());
            assert_eq!(
                text,
                expected[..text.len()],
                "captured dex fragment {index}"
            );
        }
        assert!(memory.pokedex_text(false).is_err());
        memory.preserve_oak_species_name().unwrap();
        let landmark = data
            .pokegear_landmarks
            .landmarks
            .iter()
            .find(|landmark| landmark.constant == "LANDMARK_NEW_BARK_TOWN")
            .unwrap();
        memory.format_landmark(data, landmark.id).unwrap();
        assert_eq!(
            memory.ram_text(RADIO_STRING_1).unwrap(),
            encode_radio_string("NEW BARK<BSP>TOWN@").unwrap()
        );
        assert_eq!(memory.ram_text(RADIO_MON_NAME).unwrap(), captured[..11]);
        memory.format_current_species(data).unwrap();
        assert_eq!(memory.ram_text(RADIO_STRING_1).unwrap(), captured[..11]);
        memory.format_lucky_number(42).unwrap();
        assert_eq!(
            memory.ram_text(RADIO_STRING_1).unwrap(),
            encode_radio_string("00042@").unwrap()
        );
        for category in 0..11 {
            for word in 0..3 {
                memory
                    .format_buena_password(data, category * 16 + word)
                    .unwrap();
                assert_eq!(memory.ram_text(RADIO_STRING_1).unwrap().last(), Some(&0x50));
            }
        }
        memory.format_buena_password(data, 0x30).unwrap();
        assert_eq!(
            memory.ram_text(RADIO_STRING_1).unwrap(),
            encode_radio_string("# BALL@").unwrap()
        );
        assert!(memory.format_buena_password(data, 0xb0).is_err());
        assert!(memory.format_buena_password(data, 0x03).is_err());
        let trainer_trace: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tools/asm-oracle/fixtures/radio-program-places-people.json"
        ))
        .unwrap();
        let trainer_print = trainer_trace["prints"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["next_line"] == 47)
            .unwrap();
        let trainer_key = data
            .trainers
            .trainers
            .iter()
            .find(|(_, trainer)| trainer.name == "OTIS@")
            .unwrap()
            .0;
        memory.format_source_trainer(data, 48, 1).unwrap();
        assert_eq!(
            data.trainers.trainers[trainer_key].trainer_class,
            "FIREBREATHER"
        );
        for address in [RADIO_STRING_1, RADIO_STRING_2] {
            let captured = hex(trainer_print["ram_text"][address.to_string()]
                .as_str()
                .unwrap());
            let end = captured.iter().position(|byte| *byte == 0x50).unwrap() + 1;
            assert_eq!(memory.ram_text(address).unwrap(), captured[..end]);
        }
        memory.format_source_place(data, 8).unwrap();
        let place_print = trainer_trace["prints"]
            .as_array()
            .unwrap()
            .iter()
            .find(|p| p["next_line"] == 49)
            .unwrap();
        let captured_place = hex(place_print["ram_text"]["53363"].as_str().unwrap());
        let end = captured_place
            .iter()
            .position(|byte| *byte == 0x50)
            .unwrap()
            + 1;
        assert_eq!(
            memory.ram_text(RADIO_STRING_1).unwrap(),
            captured_place[..end]
        );
        let oak_trace: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tools/asm-oracle/fixtures/radio-program-oak.json"
        ))
        .unwrap();
        memory.format_oak_encounter(data, 13, 0, 2).unwrap();
        let oak_print = &oak_trace["prints"][3];
        let captured_mon = hex(oak_print["ram_text"]["53328"].as_str().unwrap());
        let captured_place = hex(oak_print["ram_text"]["53363"].as_str().unwrap());
        let end = captured_place
            .iter()
            .position(|byte| *byte == 0x50)
            .unwrap()
            + 1;
        assert_eq!(memory.ram_text(RADIO_MON_NAME).unwrap(), captured_mon[..11]);
        assert_eq!(
            memory.ram_text(RADIO_STRING_1).unwrap(),
            captured_place[..end]
        );
        let rocket = memory.label_text(data, "_RocketRadioText1").unwrap();
        let source: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../tools/asm-oracle/fixtures/radio-program-rocket.json"
        ))
        .unwrap();
        let expected = hex(source["prints"][0]["text_bytes"].as_str().unwrap());
        assert_eq!(rocket, expected[..rocket.len()]);
    }
}
