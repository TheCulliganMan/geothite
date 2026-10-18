#!/usr/bin/env python3
"""Verify repository-complete editable sources and embedded runtime model inputs."""
from rafflesia_glb import read_gloom,read_vileplume
from steelix_glb import read_steelix
from arbok_glb import read_arbok
from ekans_glb import read_ekans
from raichu_glb import read_raichu
from pathlib import Path
from model_asset_storage import read_model_json, validate_storage
from pidgeotto_glb import read_pidgeotto
from battle_model_assets import species_paths
from jigglypuff_glb import read_jigglypuff
from clefairy_glb import read_clefairy
from sentret_glb import read_sentret
from staryu_glb import read_staryu
from fossil_scythe_glb import read_kabuto,read_kabutops
from armored_bug_glb import read_pineco,read_forretress
from pitcher_glb import read_bellsprout,read_weepinbell,read_victreebel
from voltorb_glb import read_voltorb,read_electrode
from onix_glb import read_onix
from diglett_glb import read_diglett
from dugtrio_glb import read_dugtrio
from tentacle_glb import read_tentacool,read_tentacruel
from slowpoke_glb import read_slowpoke
from entei_glb import read_entei
from raikou_glb import read_raikou
from ladybird_glb import read_ledyba, read_ledian
from spider_glb import read_spinarak,read_ariados
from quagsire_glb import read_quagsire
from hoppip_glb import read_hoppip
from skiploom_glb import read_skiploom
from jumpluff_glb import read_jumpluff
from aipom_glb import read_aipom
from sunflower_glb import read_sunkern,read_sunflora
from bellossom_glb import read_bellossom
from qwilfish_glb import read_qwilfish
from wobbuffet_glb import read_wobbuffet
from dunsparce_glb import read_dunsparce
from shuckle_glb import read_shuckle
from gligar_glb import read_gligar
from yanma_glb import read_yanma
from murkrow_glb import read_murkrow
from misdreavus_glb import read_misdreavus
from bear_glb import read_teddiursa,read_ursaring
from bulldog_glb import read_snubbull,read_granbull
from hellhound_glb import read_houndour,read_houndoom
from jet_fish_glb import read_remoraid,read_octillery
from fur_pig_glb import read_swinub,read_piloswine
from sneasel_glb import read_sneasel
from skarmory_glb import read_skarmory
from smeargle_glb import read_smeargle
from moltres_glb import read_moltres
from zapdos_glb import read_zapdos
from articuno_glb import read_articuno
from mew_glb import read_mew
from mewtwo_glb import read_mewtwo
from heracross_glb import read_heracross
from pinsir_glb import read_pinsir
from scizor_glb import read_scizor
from scyther_glb import read_scyther
from starmie_glb import read_starmie
from gastly_glb import read_gastly
from haunter_glb import read_haunter
from eevee_glb import read_eevee
from jolteon_glb import read_jolteon
from flareon_glb import read_flareon
from espeon_glb import read_espeon
from umbreon_glb import read_umbreon
from vulpix_glb import read_vulpix
from ninetales_glb import read_ninetales
from venonat_glb import read_venonat
from venomoth_glb import read_venomoth
from mushroom_crab_glb import read_paras,read_parasect
from cocoon_glb import read_metapod,read_kakuna
from larva_glb import read_caterpie,read_weedle
from crab_glb import read_krabby,read_kingler
from bivalve_glb import read_shellder,read_cloyster
from slime_glb import read_grimer,read_muk,read_ditto
from seal_glb import read_seel,read_dewgong
from turtle_glb import read_wartortle,read_blastoise
from crocodile_glb import read_croconaw,read_feraligatr
from bone_keeper_glb import read_cubone,read_marowak
from tapir_glb import read_drowzee,read_hypno
from psychic_fox_glb import read_abra,read_alakazam
from kadabra_glb import read_kadabra
from gas_pair_glb import read_koffing,read_weezing
from magnet_glb import read_magnemite,read_magneton
from tangela_glb import read_tangela
from kangaskhan_glb import read_kangaskhan
from fire_horse_glb import read_ponyta,read_rapidash
from fighting_glb import read_machop,read_machoke,read_machamp
from rhino_glb import read_rhyhorn,read_rhydon
from vaporeon_glb import read_vaporeon
from celebi_glb import read_celebi
from ho_oh_glb import read_ho_oh
from lugia_glb import read_lugia
from corsola_glb import read_corsola
from mantine_glb import read_mantine
from porygon_glb import read_porygon
from porygon2_glb import read_porygon2
from angler_glb import read_chinchou,read_lanturn
from lava_glb import read_slugma,read_magcargo
from egg_fairy_glb import read_togepi,read_togetic
from hitmonlee_glb import read_hitmonlee
from hitmonchan_glb import read_hitmonchan
from hitmontop_glb import read_hitmontop
from tyrogue_glb import read_tyrogue
from smoochum_glb import read_smoochum
from jynx_glb import read_jynx
from dodo_glb import read_doduo,read_dodrio
from raptor_bird_glb import read_pidgeot,read_fearow
from feline_glb import read_meowth,read_persian
from pig_monkey_glb import read_mankey,read_primeape
from elekid_glb import read_elekid
from magby_glb import read_magby
from magmar_glb import read_magmar
from electabuzz_glb import read_electabuzz
from slow_pair_glb import read_slowbro,read_slowking
from lickitung_glb import read_lickitung
from mr_mime_glb import read_mr_mime
from egg_nurse_glb import read_chansey,read_blissey
from rock_pupa_glb import read_larvitar,read_pupitar
from tyranitar_glb import read_tyranitar
from fire_badger_glb import read_quilava,read_typhlosion
from fire_dragon_glb import read_charmeleon,read_charizard
from stone_pair_glb import read_graveler,read_golem
from nautilus_glb import read_omanyte,read_omastar
from aerodactyl_glb import read_aerodactyl
from dragon_family_glb import read_dratini,read_dragonair,read_dragonite
from goldfish_glb import read_goldeen,read_seaking,read_magikarp
from seahorse_glb import read_horsea,read_seadra,read_kingdra
from fairy_pair_glb import read_clefable,read_wigglytuff
from sand_mammal_glb import read_sandshrew,read_sandslash
from poli_frog_glb import read_poliwhirl,read_poliwrath,read_politoed
from nido_royal_glb import read_nidoqueen,read_nidoking
from nido_middle_glb import read_nidorina,read_nidorino
from nidoran_glb import read_nidoran_f,read_nidoran_m
from fire_canine_glb import read_growlithe,read_arcanine
from baby_glb import read_pichu,read_cleffa,read_igglybuff
from electric_sheep_glb import read_mareep,read_flaaffy,read_ampharos
from psychic_bird_glb import read_natu,read_xatu
from water_rabbit_glb import read_marill,read_azumarill
from elephant_glb import read_phanpy,read_donphan
from girafarig_glb import read_girafarig
from stantler_glb import read_stantler
from delibird_glb import read_delibird
from bovine_glb import read_miltank,read_tauros
from noctowl_glb import read_noctowl
from furret_glb import read_furret
from suicune_glb import read_suicune
from unown_glb import read_unown, PARTS as UNOWN_PARTS
from hoothoot_glb import read_hoothoot
from wing_insect_glb import read_butterfree,read_beedrill
from bat_glb import read_zubat,read_golbat,read_crobat
from duck_glb import read_psyduck,read_golduck
from raticate_glb import read_raticate
from rattata_glb import read_rattata
from poliwag_glb import read_poliwag
from oddish_glb import read_oddish
from pidgey_glb import read_pidgey
from geodude_glb import read_geodude
from squirtle_glb import read_squirtle
from charmander_glb import read_charmander
from bulbasaur_glb import read_bulbasaur
from pikachu_glb import read_pikachu
from snorlax_glb import read_snorlax
from lapras_glb import read_lapras
from sudowoodo_glb import read_sudowoodo
from gyarados_glb import read_gyarados
from wooper_glb import read_wooper
from meganium_glb import read_meganium
from bayleef_glb import read_bayleef
from gengar_glb import read_gengar
from chikorita_glb import read_chikorita
from cyndaquil_glb import read_cyndaquil
from totodile_glb import read_totodile
from spearow_glb import read_spearow
import re
import math

ROOT = Path(__file__).resolve().parent.parent
from johto_art_sources import load_sources
editable = list(load_sources())

models = ROOT / 'crates/crystal-voxel-view/models'
# Imported interior meshes appear in renderer tests as literal references.
# They belong to the separately installed open-model namespace, not the
# authored Pokémon/scenery catalogue counted below.
imported_mesh_references = {
    ROOT / 'assets/open-models/interiors' / (name + '.mesh.json')
    for name in ('bed_red', 'bookcase', 'chair', 'domestic_table', 'television')
}
installed_model_references = {
    'models/' + path.relative_to(ROOT / 'assets/open-models').as_posix(): path
    for path in imported_mesh_references
}

def validate_imported_mesh_reference(path):
    model = read_model_json(path)
    assert model.get('primitives'), path
    for primitive in model['primitives']:
        positions, normals = primitive['positions'], primitive['normals']
        indices, color = primitive['indices'], primitive['base_color']
        assert positions and len(positions) % 3 == 0 and len(normals) == len(positions), path
        assert all(math.isfinite(v) for v in positions + normals), path
        assert indices and len(indices) % 3 == 0, path
        assert all(type(i) is int and 0 <= i < len(positions) // 3 for i in indices), path
        assert len(color) == 4 and all(math.isfinite(v) and 0 <= v <= 1 for v in color), path

weighted_skins = {
    models / 'battle_species/scyther.glb': (read_scyther, 57),
    models / 'battle_species/starmie.glb': (read_starmie, 23),
    models / 'battle_species/gastly.glb': (read_gastly, 19),
    models / 'battle_species/haunter.glb': (read_haunter, 23),
    models / 'battle_species/eevee.glb': (read_eevee, 47),
    models / 'battle_species/jolteon.glb': (read_jolteon, 50),
    models / 'battle_species/flareon.glb': (read_flareon, 46),
    models / 'battle_species/espeon.glb': (read_espeon, 44),
    models / 'battle_species/umbreon.glb': (read_umbreon, 46),
    models / 'battle_species/vulpix.glb': (read_vulpix, 48),
    models / 'battle_species/paras.glb': (read_paras, 56),
    models / 'battle_species/metapod.glb': (read_metapod, 15),
    models / 'battle_species/caterpie.glb': (read_caterpie, 33),
    models / 'battle_species/weedle.glb': (read_weedle, 24),
    models / 'battle_species/kakuna.glb': (read_kakuna, 14),
    models / 'battle_species/parasect.glb': (read_parasect, 36),
    models / 'battle_species/krabby.glb': (read_krabby, 31),
    models / 'battle_species/kingler.glb': (read_kingler, 36),
    models / 'battle_species/shellder.glb': (read_shellder, 19),
    models / 'battle_species/cloyster.glb': (read_cloyster, 24),
    models / 'battle_species/grimer.glb': (read_grimer, 26),
    models / 'battle_species/muk.glb': (read_muk, 26),
    models / 'battle_species/ditto.glb': (read_ditto, 8),
    models / 'battle_species/seel.glb': (read_seel, 27),
    models / 'battle_species/dewgong.glb': (read_dewgong, 22),
    models / 'battle_species/wartortle.glb': (read_wartortle, 66),
    models / 'battle_species/blastoise.glb': (read_blastoise, 71),
    models / 'battle_species/croconaw.glb': (read_croconaw, 49),
    models / 'battle_species/feraligatr.glb': (read_feraligatr, 80),
    models / 'battle_species/cubone.glb': (read_cubone, 45),
    models / 'battle_species/marowak.glb': (read_marowak, 52),
    models / 'battle_species/drowzee.glb': (read_drowzee, 43),
    models / 'battle_species/hypno.glb': (read_hypno, 48),
    models / 'battle_species/abra.glb': (read_abra, 43),
    models / 'battle_species/alakazam.glb': (read_alakazam, 53),
    models / 'battle_species/kadabra.glb': (read_kadabra, 60),
    models / 'battle_species/koffing.glb': (read_koffing, 23),
    models / 'battle_species/weezing.glb': (read_weezing, 35),
    models / 'battle_species/magnemite.glb': (read_magnemite, 18),
    models / 'battle_species/magneton.glb': (read_magneton, 54),
    models / 'battle_species/tangela.glb': (read_tangela, 49),
    models / 'battle_species/kangaskhan.glb': (read_kangaskhan, 70),
    models / 'battle_species/ponyta.glb': (read_ponyta, 47),
    models / 'battle_species/rapidash.glb': (read_rapidash, 54),
    models / 'battle_species/machop.glb': (read_machop, 32),
    models / 'battle_species/machoke.glb': (read_machoke, 41),
    models / 'battle_species/machamp.glb': (read_machamp, 51),
    models / 'battle_species/rhyhorn.glb': (read_rhyhorn, 46),
    models / 'battle_species/rhydon.glb': (read_rhydon, 64),
    models / 'battle_species/venomoth.glb': (read_venomoth, 53),
    models / 'battle_species/venonat.glb': (read_venonat, 33),
    models / 'battle_species/ninetales.glb': (read_ninetales, 56),
    models / 'battle_species/vaporeon.glb': (read_vaporeon, 57),
    models / 'battle_species/scizor.glb': (read_scizor, 55),
    models / 'battle_species/pinsir.glb': (read_pinsir, 53),
    models / 'battle_species/heracross.glb': (read_heracross, 47),
    models / 'battle_species/mewtwo.glb': (read_mewtwo, 52),
    models / 'battle_species/mew.glb': (read_mew, 40),
    models / 'battle_species/celebi.glb': (read_celebi, 38),
    models / 'battle_species/ho_oh.glb': (read_ho_oh, 207),
    models / 'battle_species/lugia.glb': (read_lugia, 56),
    models / 'battle_species/articuno.glb': (read_articuno, 109),
    models / 'battle_species/zapdos.glb': (read_zapdos, 54),
    models / 'battle_species/moltres.glb': (read_moltres, 62),
    models / 'battle_species/corsola.glb': (read_corsola, 37),
    models / 'battle_species/mantine.glb': (read_mantine, 32),
    models / 'battle_species/chinchou.glb': (read_chinchou, 21),
    models / 'battle_species/lanturn.glb': (read_lanturn, 29),

    models / 'battle_species/porygon.glb': (read_porygon, 16),
    models / 'battle_species/porygon2.glb': (read_porygon2, 17),
    models / 'battle_species/togepi.glb': (read_togepi, 27),
    models / 'battle_species/togetic.glb': (read_togetic, 37),
    models / 'battle_species/remoraid.glb': (read_remoraid, 18),
    models / 'battle_species/octillery.glb': (read_octillery, 52),
    models / 'battle_species/swinub.glb': (read_swinub, 22),
    models / 'battle_species/piloswine.glb': (read_piloswine, 22),
    models / 'battle_species/sneasel.glb': (read_sneasel, 38),
    models / 'battle_species/skarmory.glb': (read_skarmory, 42),
    models / 'battle_species/smeargle.glb': (read_smeargle, 65),
    models / 'battle_species/slugma.glb': (read_slugma, 12),
    models / 'battle_species/magcargo.glb': (read_magcargo, 22),
    models / 'battle_species/hitmonlee.glb': (read_hitmonlee, 33),
    models / 'battle_species/hitmonchan.glb': (read_hitmonchan, 36),
    models / 'battle_species/hitmontop.glb': (read_hitmontop, 35),
    models / 'battle_species/tyrogue.glb': (read_tyrogue, 40),
    models / 'battle_species/smoochum.glb': (read_smoochum, 40),
    models / 'battle_species/jynx.glb': (read_jynx,45),
    models / 'battle_species/doduo.glb': (read_doduo,37),
    models / 'battle_species/pidgeot.glb': (read_pidgeot,105),
    models / 'battle_species/fearow.glb': (read_fearow,161),
    models / 'battle_species/dodrio.glb': (read_dodrio,61),
    models / 'battle_species/meowth.glb': (read_meowth,51),
    models / 'battle_species/persian.glb': (read_persian,48),
    models / 'battle_species/mankey.glb': (read_mankey,52),
    models / 'battle_species/primeape.glb': (read_primeape,59),
    models / 'battle_species/elekid.glb': (read_elekid, 32),
    models / 'battle_species/magby.glb': (read_magby, 38),
    models / 'battle_species/magmar.glb': (read_magmar, 55),
    models / 'battle_species/electabuzz.glb': (read_electabuzz, 70),
    models / 'battle_species/slowbro.glb': (read_slowbro, 72),
    models / 'battle_species/lickitung.glb': (read_lickitung, 33),
    models / 'battle_species/mr__mime.glb': (read_mr_mime, 67),
    models / 'battle_species/chansey.glb': (read_chansey, 24),
    models / 'battle_species/blissey.glb': (read_blissey, 38),
    models / 'battle_species/horsea.glb': (read_horsea, 36),
    models / 'battle_species/goldeen.glb': (read_goldeen, 36),
    models / 'battle_species/larvitar.glb': (read_larvitar, 45),
    models / 'battle_species/pupitar.glb': (read_pupitar, 28),
    models / 'battle_species/tyranitar.glb': (read_tyranitar, 70),
    models / 'battle_species/quilava.glb': (read_quilava, 68),
    models / 'battle_species/typhlosion.glb': (read_typhlosion, 77),
    models / 'battle_species/charmeleon.glb': (read_charmeleon, 52),
    models / 'battle_species/charizard.glb': (read_charizard, 65),
    models / 'battle_species/graveler.glb': (read_graveler, 40),
    models / 'battle_species/golem.glb': (read_golem, 77),
    models / 'battle_species/omanyte.glb': (read_omanyte, 32),
    models / 'battle_species/omastar.glb': (read_omastar, 43),
    models / 'battle_species/aerodactyl.glb': (read_aerodactyl, 56),
    models / 'battle_species/dratini.glb': (read_dratini, 17),
    models / 'battle_species/dragonair.glb': (read_dragonair, 21),
    models / 'battle_species/dragonite.glb': (read_dragonite, 60),
    models / 'battle_species/seaking.glb': (read_seaking, 70),
    models / 'battle_species/magikarp.glb': (read_magikarp, 30),
    models / 'battle_species/seadra.glb': (read_seadra, 34),
    models / 'battle_species/kingdra.glb': (read_kingdra, 47),
    models / 'battle_species/clefable.glb': (read_clefable, 32),
    models / 'battle_species/wigglytuff.glb': (read_wigglytuff, 24),
    models / 'battle_species/sandshrew.glb': (read_sandshrew, 58),
    models / 'battle_species/sandslash.glb': (read_sandslash, 53),
    models / 'battle_species/poliwhirl.glb': (read_poliwhirl, 25),
    models / 'battle_species/poliwrath.glb': (read_poliwrath, 27),
    models / 'battle_species/politoed.glb': (read_politoed, 36),
    models / 'battle_species/nidoqueen.glb': (read_nidoqueen, 61),
    models / 'battle_species/nidoking.glb': (read_nidoking, 70),
    models / 'battle_species/nidorina.glb': (read_nidorina, 59),
    models / 'battle_species/nidorino.glb': (read_nidorino, 74),
    models / 'battle_species/nidoran_f.glb': (read_nidoran_f, 65),
    models / 'battle_species/nidoran_m.glb': (read_nidoran_m, 53),
    models / 'battle_species/growlithe.glb': (read_growlithe, 57),
    models / 'battle_species/arcanine.glb': (read_arcanine, 81),
    models / 'battle_species/slowking.glb': (read_slowking, 60),
    models / 'battle_species/pichu.glb': (read_pichu, 24),
    models / 'battle_species/cleffa.glb': (read_cleffa, 17),
    models / 'battle_species/igglybuff.glb': (read_igglybuff, 19),

    models / 'battle_species/mareep.glb': (read_mareep, 26),
    models / 'battle_species/flaaffy.glb': (read_flaaffy, 33),
    models / 'battle_species/ampharos.glb': (read_ampharos, 26),
    models / 'battle_species/natu.glb': (read_natu, 46),
    models / 'battle_species/xatu.glb': (read_xatu, 49),
    models / 'battle_species/marill.glb': (read_marill, 19),
    models / 'battle_species/azumarill.glb': (read_azumarill, 25),
    models / 'battle_species/phanpy.glb': (read_phanpy, 32),
    models / 'battle_species/donphan.glb': (read_donphan, 49),
    models / 'battle_species/girafarig.glb': (read_girafarig, 72),
    models / 'battle_species/stantler.glb': (read_stantler, 51),
    models / 'battle_species/delibird.glb': (read_delibird, 28),
    models / 'battle_species/miltank.glb': (read_miltank, 45),
    models / 'battle_species/tauros.glb': (read_tauros, 42),
    models / 'battle_species/houndour.glb': (read_houndour, 44),
    models / 'battle_species/houndoom.glb': (read_houndoom, 67),
    models / 'battle_species/snubbull.glb': (read_snubbull, 39),
    models / 'battle_species/granbull.glb': (read_granbull, 42),
    models / 'battle_species/teddiursa.glb': (read_teddiursa, 37),
    models / 'battle_species/ursaring.glb': (read_ursaring, 48),
    models / 'battle_species/misdreavus.glb': (read_misdreavus, 41),
    models / 'battle_species/murkrow.glb': (read_murkrow, 62),
    models / 'battle_species/yanma.glb': (read_yanma, 66),
    models / 'battle_species/gligar.glb': (read_gligar, 32),
    models / 'battle_species/shuckle.glb': (read_shuckle, 27),
    models / 'battle_species/dunsparce.glb': (read_dunsparce, 19),
    models / 'battle_species/wobbuffet.glb': (read_wobbuffet, 19),
    models / 'battle_species/qwilfish.glb': (read_qwilfish, 36),
    models / 'battle_species/bellossom.glb': (read_bellossom, 45),
    models / 'battle_species/sunkern.glb': (read_sunkern, 18),
    models / 'battle_species/sunflora.glb': (read_sunflora, 30),
    models / 'battle_species/aipom.glb': (read_aipom, 50),
    models / 'battle_species/jumpluff.glb': (read_jumpluff, 106),
    models / 'battle_species/skiploom.glb': (read_skiploom, 24),
    models / 'battle_species/hoppip.glb': (read_hoppip, 21),
    models / 'battle_species/ledyba.glb': (read_ledyba, 36),
    models / 'battle_species/ledian.glb': (read_ledian, 38),
    models / 'battle_species/spinarak.glb': (read_spinarak, 43),
    models / 'battle_species/ariados.glb': (read_ariados, 41),
    models / 'battle_species/quagsire.glb': (read_quagsire, 22),
    models / 'battle_species/noctowl.glb': (read_noctowl, 52),
    models / 'battle_species/furret.glb': (read_furret, 33),
    models / 'battle_species/suicune.glb': (read_suicune, 47),
    models / 'battle_species/raikou.glb': (read_raikou, 62),
    models / 'battle_species/entei.glb': (read_entei, 59),
    models / 'battle_species/slowpoke.glb': (read_slowpoke, 40),
    models / 'battle_species/diglett.glb': (read_diglett, 15),
    models / 'battle_species/dugtrio.glb': (read_dugtrio, 33),
    models / 'battle_species/tentacool.glb': (read_tentacool,13),
    models / 'battle_species/tentacruel.glb': (read_tentacruel,23),
    models / 'battle_species/onix.glb': (read_onix, 36),
    models / 'battle_species/voltorb.glb': (read_voltorb, 8),
    models / 'battle_species/electrode.glb': (read_electrode, 10),
    models / 'battle_species/bellsprout.glb': (read_bellsprout, 14),
    models / 'battle_species/weepinbell.glb': (read_weepinbell, 20),
    models / 'battle_species/victreebel.glb': (read_victreebel, 25),
    models / 'battle_species/gloom.glb': (read_gloom, 30),
    models / 'battle_species/vileplume.glb': (read_vileplume, 34),
    models / 'battle_species/steelix.glb': (read_steelix, 48),
    models / 'battle_species/arbok.glb': (read_arbok, 25),
    models / 'battle_species/ekans.glb': (read_ekans, 16),
    models / 'battle_species/raichu.glb': (read_raichu, 39),

    models / 'battle_species/pineco.glb': (read_pineco, 54),
    models / 'battle_species/forretress.glb': (read_forretress, 48),
    models / 'battle_species/kabuto.glb': (read_kabuto, 23),
    models / 'battle_species/kabutops.glb': (read_kabutops, 42),
    models / 'battle_species/staryu.glb': (read_staryu, 10),
    models / 'battle_species/sentret.glb': (read_sentret, 38),
    models / 'battle_species/hoothoot.glb': (read_hoothoot, 38),
    models / 'battle_species/butterfree.glb': (read_butterfree, 59),
    models / 'battle_species/beedrill.glb': (read_beedrill, 42),
    models / 'battle_species/zubat.glb': (read_zubat, 28),
    models / 'battle_species/golbat.glb': (read_golbat, 32),
    models / 'battle_species/crobat.glb': (read_crobat, 28),
    models / 'battle_species/psyduck.glb': (read_psyduck, 28),
    models / 'battle_species/golduck.glb': (read_golduck, 45),
    models / 'battle_species/raticate.glb': (read_raticate, 63),
    models / 'battle_species/rattata.glb': (read_rattata, 42),
    models / 'battle_species/poliwag.glb': (read_poliwag, 19),
    models / 'battle_species/oddish.glb': (read_oddish, 19),
    models / 'battle_species/pidgey.glb': (read_pidgey, 43),
    models / 'battle_species/geodude.glb': (read_geodude, 29),
    models / 'battle_species/squirtle.glb': (read_squirtle, 45),
    models / 'battle_species/charmander.glb': (read_charmander, 43),
    models / 'battle_species/bulbasaur.glb': (read_bulbasaur, 44),
    models / 'battle_species/pikachu.glb': (read_pikachu, 40),
    models / 'battle_species/snorlax.glb': (read_snorlax, 31),
    models / 'battle_species/lapras.glb': (read_lapras, 49),
    models / 'battle_species/sudowoodo.glb': (read_sudowoodo, 33),
    models / 'battle_species/gyarados.glb': (read_gyarados, 40),
    models / 'battle_species/wooper.glb': (read_wooper, 22),
    models / 'battle_species/meganium.glb': (read_meganium, 33),
    models / 'battle_species/bayleef.glb': (read_bayleef, 33),
    models / 'actor_props/icon_jigglypuff.glb': (read_jigglypuff, 22),
    models / 'actor_props/icon_clefairy.glb': (read_clefairy, 35),
    models / 'actor_props/battle_chikorita.glb': (read_chikorita, 19),
    models / 'actor_props/battle_cyndaquil.glb': (read_cyndaquil, 30),
    models / 'actor_props/battle_totodile.glb': (read_totodile, 30),
    models / 'battle_species/gengar.glb': (read_gengar, 27),
    models / 'battle_species/spearow.glb': (read_spearow, 28),
}
weighted_skins.update({models / 'battle_species' / (species+'.glb'): (read_unown,len(parts)) for species,parts in UNOWN_PARTS.items()})
canonical_model_names = {
    models / 'actor_props/icon_clefairy.glb': 'clefairy',
    models / 'actor_props/icon_jigglypuff.glb': 'jigglypuff',
}
model_files = sorted([*models.rglob('*.mesh.json'), *models.rglob('*.rig.json'), *models.rglob('*.glb')])
model_counts = {}
species_files = species_paths(models / 'battle_species', include_starters=True)
validate_storage(models)
assert model_files
for path in model_files:
    if path.suffix == '.glb':
        if path in weighted_skins:
            reader, part_count = weighted_skins[path]
            model = reader(path)
            assert model['name'] == canonical_model_names.get(path, path.stem) and len(model['primitives']) == part_count, path
            model_counts[path] = 1
            continue
        if path == models / 'battle_species/pidgeotto.glb':
            assert len(read_pidgeotto(path)['primitives']) == 34
            model_counts[path] = 1
            continue
        raise AssertionError(f'unregistered GLB: {path}')
    model_counts[path] = 1
    model = read_model_json(path)
    assert 'primitives' in model or 'joints' in model, path
    if 'joints' in model:
        versions = (1, 2) if path.parent.name == 'johto_characters' else (1,)
        assert model['version'] in versions and len(model['joints']) == 16, path
    assert not path.read_bytes().startswith(b'version https://git-lfs'), path

references = 0
for source in (ROOT / 'crates/crystal-voxel-view/src').rglob('*.rs'):
    # Literal paths may be arguments to a small lazy-loader macro. Inspect the
    # actual paths rather than depending on rustfmt's include_str layout.
    for relative in re.findall(r'"([^"\n]*models/[^"\n]+\.(?:json|glb)(?:\.include\.rs)?)"', source.read_text()):
        # Test readers may build a filesystem path with format!; that is not
        # an embedded asset reference. Production include paths stay literal.
        if '{' in relative or '}' in relative:
            assert source.name.endswith('_tests.rs'), (source, relative)
            continue
        # The external loader's policy tests deliberately contain invalid and
        # absent runtime keys. Only its literal embedded fixture is a file.
        if source.name == 'open_models.rs' and relative.startswith('models/'):
            continue
        if (source.name in ('interior_models.rs', 'new_bark_models.rs', 'foliage.rs')
                and relative in installed_model_references):
            validate_imported_mesh_reference(installed_model_references[relative])
            continue
        base = models.parent if relative.startswith('models/') else source.parent
        path = (base / relative.removesuffix('.include.rs')).resolve()
        assert path.is_file(), (source, relative)
        if path not in model_files:
            if path in imported_mesh_references:
                validate_imported_mesh_reference(path)
            else:
                assert path in (models / 'battle/arenas.json',), path
                read_model_json(path)
            continue
        if not (source.name == 'johto_actor_props.rs' and relative.endswith('.include.rs')):
            references += model_counts[path]
    # The prop catalog uses a declarative macro so source identities and
    # embedded mesh paths cannot drift. Verify every concrete expansion too.
    if source.name == 'johto_actor_props.rs':
        declaration = re.search(r'authored_props!\s*\{(.*?)\n\}', source.read_text(), re.S)
        assert declaration, 'missing source-scoped actor model catalog'
        for label in re.findall(r'=>\s*"([^"\n]+)"', declaration.group(1)):
            if label in ('chris_bike','kris_bike'):
                assert not (models / 'actor_props' / (label + '.mesh.json')).exists()
                continue  # Native bicycle geometry, with the shared weighted rider.
            if label == 'icon_unown':
                path = models / 'battle_species/unown_i.glb'
                assert path in weighted_skins and path in model_files
                assert not (models / 'actor_props/icon_unown.mesh.json').exists()
                references += 1
                continue
            if label in ('icon_staryu','icon_voltorb','big_onix','icon_diglett','slowpoke','icon_slowpoke','entei','raikou','suicune'):
                path = models / 'battle_species' / (label.removeprefix('icon_').removeprefix('big_') + '.glb')
                assert path in weighted_skins and path in model_files
                assert not (models / 'actor_props' / (label + '.mesh.json')).exists()
                references += 1
                continue
            path = models / 'actor_props' / (label + '.mesh.json')
            skin = path.with_name(label + '.glb')
            if skin in weighted_skins:
                assert not path.exists(), 'canonical skin must not retain duplicate JSON geometry'
                path = skin
            assert path.is_file(), (source, label)
            assert path in model_files, path
            references += 1
    if source.name == 'battle_species_models.rs':
        declaration = re.search(r'species_models!\s*\{(.*?)\n\}', source.read_text(), re.S)
        assert declaration, 'missing exact battle species catalog'
        for label in re.findall(r'=>\s*"([^"\n]+)"', declaration.group(1)):
            if re.fullmatch(r'unown_[a-z]', label):
                path = models / 'battle_species' / (label + '.glb')
                assert path in weighted_skins, path
            else:
                path = species_files[label]
            assert path.is_file(), (source, label)
            assert path in model_files, path
            references += 1
model_count = sum(model_counts.values())
assert references >= model_count - 3  # remaining authored props may use dedicated weighted species skins
print(f'Validated {len(editable)} complete editable Blender sources, '
      f'{model_count} runtime models in {len(model_files)} files and {references} embedded references')
