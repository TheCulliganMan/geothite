#!/usr/bin/env python3
"""Hand-furnish frozen v09 St Paul rooms; never writes native pack data.

Only explicit new-room rows change. Native house furniture is reused verbatim.
The companion proof rejects non-terrain drift and occupied or severed anchors.
"""
import copy
import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BASE = ROOT / 'planning/minnesota'
PREVIOUS = BASE / 'renders/st-paul/tight-20261027/v09'
TOWNS = BASE / 'towns'

# Each six-row room is composed individually. No generated scatter or palette edits.
ROWS = {'FairAgricultureHall': ['04 05 03 05 03 05 04 1d',
                         '0f 0f 0f 0f 0f 2c 0f 0f',
                         '0f 0f 0f 0f 0f 0f 0f 2c',
                         '0f 0f 2f 0f 30 0f 2c 2c',
                         '0f 0f 0f 0f 30 0f 2c 2c',
                         '06 0b 0f 0f 0f 0f 0f 07'],
 'FairExhibitionHall': ['04 05 03 05 03 05 03 05',
                        '0f 0f 0f 0f 0f 1f 1f 27',
                        '0f 0f 0f 0f 0f 0f 0f 27',
                        '0f 0f 14 15 0f 2d 2d 27',
                        '0f 0f 18 19 0f 0f 2e 0f',
                        '06 0b 0f 0f 0f 0f 0f 07'],
 'FairBreederBarn': ['04 05 03 05 03 05 04 05',
                     '0f 0f 0f 0f 0f 14 15 0f',
                     '0f 1a 0f 0f 0f 18 19 1a',
                     '0f 0f 0f 0f 0f 0f 0f 1a',
                     '0f 0f 0f 0f 0f 2f 2c 0f',
                     '06 0b 0f 0f 0f 0f 0f 07'],
 'CapitolArchive': ['04 05 03 05 04 04 04 1d',
                    '0f 0f 0f 0f 0f 0f 0f 0f',
                    '0f 0f 0f 1a 0f 1a 0f 1a',
                    '0f 0f 0f 1a 0f 1a 0f 1a',
                    '0f 0f 0f 0f 0f 0f 0f 1a',
                    '06 0b 0f 0f 0f 1a 1a 1a'],
 'CapitolLibrary': ['04 05 03 05 04 04 04 1d',
                    '0f 0f 0f 0f 0f 0f 0f 0f',
                    '0f 0f 0f 0f 1a 0f 1a 1a',
                    '0f 0f 2d 0f 1a 0f 1a 1a',
                    '0f 0f 0f 0f 0f 0f 0f 1a',
                    '06 0b 0f 0f 1a 1a 0f 07'],
 'CivicMuseum': ['04 05 03 05 03 05 04 1d',
                 '0f 0f 0f 0f 0f 1f 0f 0f',
                 '0f 0f 0f 0f 0f 0f 0f 1f',
                 '0f 0f 1f 0f 1f 0f 1f 1f',
                 '0f 0f 0f 0f 0f 0f 0f 0f',
                 '06 0b 0f 0f 0f 0f 0f 07'],
 'SummitExhibitionClub': ['04 05 03 05 03 05 04 05',
                          '0f 0f 0f 0f 0f 01 02 27',
                          '0f 0f 0f 0f 0f 0c 0d 27',
                          '0f 0f 2d 0f 0f 0f 0f 27',
                          '0f 0f 0f 0f 0f 0f 0f 0f',
                          '06 0b 0f 0f 1a 0f 0f 07'],
 'WestSeventhBooks': ['04 05 03 05 04 04 04 1d',
                      '0f 0f 0f 0f 0f 0f 0f 0f',
                      '0f 1a 0f 0f 1a 0f 1a 1a',
                      '0f 0f 0f 0f 1a 0f 1a 1a',
                      '0f 0f 14 15 0f 0f 0f 1a',
                      '06 0b 18 19 0f 0f 0f 07'],
 'ArchiveAnnex': ['04 05 03 05 04 04 04 1d',
                  '0f 0f 0f 0f 0f 0f 0f 0f',
                  '0f 1a 0f 1a 0f 0f 1a 1a',
                  '0f 0f 0f 0f 0f 0f 1a 1a',
                  '0f 0f 0f 1a 0f 0f 0f 1a',
                  '06 0b 0f 0f 0f 0f 0f 07'],
 'EastWorkshop': ['04 05 03 05 04 04 04 05',
                  '0f 0f 0f 0f 0f 2d 2d 27',
                  '0f 0f 0f 0f 0f 0f 0f 27',
                  '0f 0f 2d 0f 0f 0f 0f 27',
                  '0f 0f 0f 0f 0f 14 15 0f',
                  '06 0b 0f 0f 0f 18 19 07'],
 'CommunityHall': ['04 05 03 05 03 05 03 1d',
                   '0f 0f 0f 0f 0f 0f 0f 0f',
                   '1a 0f 0f 14 15 0f 0f 0f',
                   '1a 0f 0f 21 22 0f 1a 0f',
                   '0f 0f 0f 18 19 0f 0f 0f',
                   '06 0b 0f 0f 0f 0f 0f 07'],
 'FreightOffice': ['04 05 03 05 04 04 04 1d',
                   '0f 0f 0f 0f 0f 0f 0f 0f',
                   '0f 1a 0f 0f 14 15 0f 1a',
                   '0f 0f 1a 0f 21 22 0f 1a',
                   '0f 0f 0f 0f 18 19 0f 1a',
                   '06 0b 0f 0f 0f 0f 0f 07'],
 'AnglerHouse': ['04 05 03 05 03 05 04 05',
                 '0f 0f 0f 0f 0f 1a 0f 0f',
                 '0f 0f 14 15 0f 0f 0f 1a',
                 '0f 0f 18 19 0f 0f 0f 1a',
                 '0f 0f 0f 0f 0f 2d 0f 0f',
                 '06 0b 0f 0f 0f 0f 0f 07'],
 'WestsideCommunity': ['04 05 03 05 03 05 04 1d',
                       '0f 0f 0f 0f 0f 01 02 0f',
                       '0f 2d 0f 0f 0f 0c 0d 1a',
                       '0f 0f 0f 0f 0f 0f 0f 1a',
                       '0f 0f 0f 14 15 0f 0f 0f',
                       '06 0b 0f 18 19 0f 0f 07'],
 'WestsideArena': ['04 05 03 05 03 05 03 05',
                   '0f 0f 0f 0f 0f 2d 2d 27',
                   '0f 0f 0f 0f 0f 0f 0f 27',
                   '0f 0f 2d 0f 0f 0f 2d 27',
                   '0f 0f 0f 0f 0f 0f 2e 0f',
                   '06 0b 0f 0f 0f 0f 0f 07'],
 'SouthLandingInn': ['04 05 03 05 03 05 04 1d',
                     '0f 0f 0f 0f 0f 1a 0f 0f',
                     '0f 0f 0f 14 15 0f 0f 1a',
                     '0f 0f 0f 18 19 0f 14 1a',
                     '0f 0f 0f 0f 0f 0f 18 19',
                     '06 0b 0f 0f 0f 0f 0f 07'],
 'SouthLandingSurvey': ['04 05 03 05 04 04 04 05',
                        '0f 0f 0f 0f 0f 2d 2d 0f',
                        '0f 0f 0f 0f 0f 0f 0f 1a',
                        '0f 0f 14 15 0f 1a 0f 1a',
                        '0f 0f 18 19 0f 1a 0f 0f',
                        '06 0b 0f 0f 0f 0f 0f 07']}

def main():
    contract = json.loads((PREVIOUS / 'MnStPaul.tight.expansion.contract.json').read_text())
    dossier = json.loads((PREVIOUS / 'MnStPaul.tight.candidate.json').read_text())
    for room in contract['rooms']:
        key = room['id'].removeprefix('MnStPaul')
        if key in ROWS:
            room['rows'] = ROWS[key]
    dossier['new_rooms'] = copy.deepcopy(contract['rooms'])
    for name, value in [('MnStPaul.tight.expansion.contract.json', contract), ('MnStPaul.tight.candidate.json', dossier)]:
        (TOWNS / name).write_text(json.dumps(value, indent=2, ensure_ascii=False) + '\n')
    print('Refined explicit rows in', len(ROWS), 'large rooms; all other authoring values retained.')

if __name__ == '__main__':
    main()
