// Source identities and sparse masks only; no source pixels or game content.
const MAPS: &[&str] = &[
    "CeladonPokecenter2FBeta",
    "CeruleanPokecenter2FBeta",
    "CinnabarPokecenter2FBeta",
    "FuchsiaPokecenter2FBeta",
    "LavenderPokecenter2FBeta",
    "PewterPokecenter2FBeta",
    "Pokecenter2F",
    "Route10Pokecenter2FBeta",
    "SaffronPokecenter2FBeta",
    "VermilionPokecenter2FBeta",
    "ViridianPokecenter2FBeta",
];
const NETWORKS: &[Network] = &[
    Network {
        anchor: [5, 0],
        width: 4,
        height: 9,
        fingerprint: 0x672a88e9b1c87a97,
        rows: &[6, 6, 6, 6, 6, 6, 6, 6, 0],
        shells: &[[1, 0, 2, 8]],
        record_sign: false,
    },
    Network {
        anchor: [13, 0],
        width: 4,
        height: 9,
        fingerprint: 0x1699ed2747d5e75f,
        rows: &[6, 6, 6, 6, 6, 6, 6, 6, 0],
        shells: &[[1, 0, 2, 8]],
        record_sign: true,
    },
    Network {
        anchor: [21, 0],
        width: 11,
        height: 9,
        fingerprint: 0x329df3471d9275b2,
        rows: &[1950, 1950, 1950, 1950, 1950, 1950, 1542, 1542, 0],
        shells: &[[1, 0, 2, 8], [3, 0, 2, 6], [7, 0, 2, 6], [9, 0, 2, 8]],
        record_sign: false,
    },
];
