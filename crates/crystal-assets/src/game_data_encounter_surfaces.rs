//! Keep optional land and water tables independent at the native boundary.
use crystal_core::world::encounters::{EncounterSurface, WildEncounterData};

/// An absent component with no positive rate behaves like a map with no wilds.
/// Passing `None` lets the native chooser retain its zero-rate RNG behavior.
/// Inconsistent positive rates remain visible to ordinary validation/preflight.
pub(crate) fn for_surface(
    data: Option<&WildEncounterData>,
    surface: Option<EncounterSurface>,
) -> Option<&WildEncounterData> {
    let data = data?;
    let absent = match surface {
        Some(EncounterSurface::Water) => {
            data.water.is_none() && data.water_rate.is_none_or(|rate| rate == 0)
        }
        Some(EncounterSurface::Grass | EncounterSurface::Rock) => {
            data.grass.is_none()
                && data
                    .grass_rates
                    .as_ref()
                    .is_none_or(|rates| rates.values().all(|rate| *rate == 0))
        }
        None => false,
    };
    (!absent).then_some(data)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crystal_core::world::encounters::{
        EncounterSurface, WildEncounterData, WildEncounterTable,
    };

    use super::for_surface;

    #[test]
    fn surf_only_cave_keeps_water_and_has_no_land_transaction() {
        let mut data = WildEncounterData {
            water: Some(WildEncounterTable::default()),
            water_rate: Some(5),
            ..Default::default()
        };
        assert_eq!(
            for_surface(Some(&data), Some(EncounterSurface::Water)),
            Some(&data)
        );
        for surface in [EncounterSurface::Grass, EncounterSurface::Rock] {
            assert!(for_surface(Some(&data), Some(surface)).is_none());
        }
        data.grass_rates = Some(BTreeMap::from([("day".into(), 0)]));
        assert!(for_surface(Some(&data), Some(EncounterSurface::Grass)).is_none());
    }

    #[test]
    fn land_only_cave_keeps_its_table_and_has_no_water_transaction() {
        let mut data = WildEncounterData {
            grass: Some(WildEncounterTable::default()),
            grass_rates: Some(BTreeMap::from([("day".into(), 10)])),
            ..Default::default()
        };
        assert_eq!(
            for_surface(Some(&data), Some(EncounterSurface::Grass)),
            Some(&data)
        );
        assert!(for_surface(Some(&data), Some(EncounterSurface::Water)).is_none());
        data.water_rate = Some(0);
        assert!(for_surface(Some(&data), Some(EncounterSurface::Water)).is_none());
    }

    #[test]
    fn missing_tables_with_positive_rates_still_reach_strict_validation() {
        let data = WildEncounterData {
            water_rate: Some(5),
            grass_rates: Some(BTreeMap::from([("day".into(), 0), ("night".into(), 10)])),
            ..Default::default()
        };
        assert_eq!(
            for_surface(Some(&data), Some(EncounterSurface::Grass)),
            Some(&data)
        );
        assert_eq!(
            for_surface(Some(&data), Some(EncounterSurface::Water)),
            Some(&data)
        );
    }

    #[test]
    fn no_data_or_nonencounter_surface_keeps_existing_semantics() {
        let data = WildEncounterData::default();
        assert!(for_surface(None, Some(EncounterSurface::Grass)).is_none());
        assert_eq!(for_surface(Some(&data), None), Some(&data));
    }
}
