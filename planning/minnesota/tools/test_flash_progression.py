#!/usr/bin/env python3
"""Regression tests for the compiled-pack Flash progression audit."""
import contextlib
import io
import unittest

import check_flash_progression


def map_room(name, palette='PALETTE_NITE', selector=18, warps=(), connections=()):
    return {
        'attributes': {
            'map_constant': name,
            'palette': palette,
            'phone_service': selector,
            'connections': list(connections),
        },
        'events': {'warps': [{'target_map': target} for target in warps]},
        'objects': [],
        'scripts': {},
    }


def candidate():
    gallery_id = 'MN_DRIFTLESS_LAMP_GALLERY'
    hollow = map_room('MnR01Cave1F', warps=('MnR01C00R00', 'MnR01Cave1F', gallery_id))
    hollow['objects'] = [{'script': 'MN_R01_HOLLOW_GUIDEFieldHM'}]
    hollow['scripts'] = {
        'MN_R01_HOLLOW_GUIDEFieldHM': [
            {'command': 'checkflag', 'args': ['ENGINE_ZEPHYRBADGE']},
            {'command': 'iffalse', 'args': ['Denied']},
            {'command': 'verbosegiveitem', 'args': ['HM_FLASH', '1']},
        ],
        'Denied': [{'command': 'writetext', 'args': ['DeniedText']}],
    }
    gallery = map_room(gallery_id, 'dark', 20, ('MnR01Cave1F', gallery_id))
    route = map_room('MnR01C00R00', connections=(
        {'direction': 'east', 'target_map': 'MnWinona'},
        {'direction': 'west', 'target_map': 'MnRochester'},
    ))
    maps = {'MnR01Cave1F': hollow, 'MnDriftlessLampGallery': gallery,
            'MnR01C00R00': route}
    metadata = {room['attributes']['map_constant']: {
        'name': name, 'groupName': 'MINNESOTA_DRIFTLESS',
        'phoneService': room['attributes']['phone_service'],
    } for name, room in maps.items()}
    return {'data': {
        'maps': maps,
        'runtime_map_metadata': metadata,
        'field_moves': {'flash': {'move_id': 'FLASH',
                                  'badge': {'region': 'johto', 'index': 0},
                                  'engine_flag': 'STATUSFLAGS_FLASH'}},
        'items': {'HM_FLASH': {'tmhm_move': 'FLASH', 'consumable': False}},
        'pokemon': {'STRIPILLAR': {'tmhm_learnset': ['FLASH']},
                    'TESSLARV': {'tmhm_learnset': ['FLASH']}},
        'wild_encounters': {'MnR01C00R00': {
            'grass': {'morning': [{'species': 'STRIPILLAR'}],
                      'day': [{'species': 'STRIPILLAR'}], 'night': []},
            'zones': [{'id': 'MN_HABITAT_DRIFTLESS_SEEP',
                       'grass': {'night': [{'species': 'TESSLARV'}]}}],
        }},
    }}


class FlashProgressionTests(unittest.TestCase):
    def setUp(self):
        self.pack = candidate()

    def audit(self):
        with contextlib.redirect_stdout(io.StringIO()):
            check_flash_progression.check(self.pack)

    def test_lit_route_and_optional_dark_gallery_are_accepted(self):
        self.audit()

    def test_dark_hollow_or_disagreeing_runtime_selector_is_rejected(self):
        for component in ('palette', 'runtime'):
            with self.subTest(component=component):
                self.pack = candidate()
                if component == 'palette':
                    self.pack['data']['maps']['MnR01Cave1F']['attributes']['palette'] = 'dark'
                else:
                    self.pack['data']['runtime_map_metadata']['MnR01Cave1F']['phoneService'] = 20
                with self.assertRaisesRegex(ValueError, 'darkness selector disagree'):
                    self.audit()

    def test_new_dark_region_map_is_rejected(self):
        self.pack['data']['maps']['MnAnotherCave'] = map_room('MnAnotherCave', 'dark', 20)
        self.pack['data']['runtime_map_metadata']['MnAnotherCave'] = {
            'name': 'MnAnotherCave', 'groupName': 'MINNESOTA_TEST', 'phoneService': 20,
        }
        with self.assertRaisesRegex(ValueError, 'unreviewed Minnesota dark maps'):
            self.audit()

    def test_gallery_cannot_become_a_through_route(self):
        gallery = self.pack['data']['maps']['MnDriftlessLampGallery']
        gallery['events']['warps'].append({'target_map': 'MnRochester'})
        with self.assertRaisesRegex(ValueError, 'unreviewed exit'):
            self.audit()

    def test_surface_route_and_gallery_entrance_cannot_change_unnoticed(self):
        for mutation in ('surface', 'entrance'):
            with self.subTest(mutation=mutation):
                self.pack = candidate()
                route = self.pack['data']['maps']['MnR01C00R00']
                if mutation == 'surface':
                    route['attributes']['connections'].pop()
                else:
                    route['events']['warps'].append({'target_map': 'MN_DRIFTLESS_LAMP_GALLERY'})
                with self.assertRaises(ValueError):
                    self.audit()

    def test_flash_requires_badge_before_gift(self):
        script = self.pack['data']['maps']['MnR01Cave1F']['scripts']['MN_R01_HOLLOW_GUIDEFieldHM']
        script[1]['command'] = 'iftrue'
        with self.assertRaisesRegex(ValueError, 'checks the Care Badge'):
            self.audit()

    def test_flash_learner_is_available_in_each_time_period(self):
        for mutation in ('remove_period', 'remove_night_learner'):
            with self.subTest(mutation=mutation):
                self.pack = candidate()
                encounter = self.pack['data']['wild_encounters']['MnR01C00R00']
                if mutation == 'remove_period':
                    del encounter['grass']['night']
                else:
                    encounter['zones'][0]['grass']['night'] = []
                with self.assertRaises(ValueError):
                    self.audit()


if __name__ == '__main__':
    unittest.main()
