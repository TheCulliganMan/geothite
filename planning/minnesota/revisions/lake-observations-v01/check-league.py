#!/usr/bin/env python3
"""Trace the eight-badge desk, optional invitation and five battle bindings in a built edition."""
import argparse
from itertools import product
import json
from pathlib import Path
import sys

HERE = Path(__file__).resolve().parent
REGION = HERE.parents[1]
sys.path.insert(0, str(REGION / 'tools'))
from audit_story_flow import native_pages, require
from render_reference import load_pack

LOBBY = 'MnLeagueLobby'
CASEY = 'MnLeagueChampion'
ACTIVE = 'EVENT_MN_LEAGUE_RUN_ACTIVE'
TITLE = 'EVENT_MN_LEAGUE_CHAMPION'
CLEAR = 'EVENT_MN_SURVEY_RESTORED'
RUN = 'wMnLeagueRun'
CLUES = (
    ('EVENT_MN_ARC_LINK_SENT', 'MnElyGoToIntake'),
    ('EVENT_MN_SURVEY_CHIEF_DEFEATED', 'MnElyReadPumpTimes'),
    ('EVENT_MN_LIGHTHOUSE_SIGNAL', 'MnElyFindShelter'),
    ('EVENT_MN_DEPOT_CLEARED', 'MnElyCallHarbor'),
    ('EVENT_MN_MINE_INSTRUMENT_RECOVERED', 'MnElyBringMeter'),
    ('EVENT_MN_HEADWATERS_REPORT', 'MnElyLookForFinn'),
    ('EVENT_MN_HEADWATERS_READING', 'MnElyTakeReading'),
)
ROOMS = ['MnLeagueRoom1', 'MnLeagueRoom2', 'MnLeagueRoom3', 'MnLeagueRoom4', CASEY]


def visit(room, start, flags, wins=0, yes=False):
    """Follow the native instructions to an exit, first battle or scripted warp."""
    name, index, value = start, 0, None
    pages, choices, writes = [], 0, []
    for _ in range(96):
        row = room['scripts'][name][index]
        index += 1
        op, args = row['command'], row['args']
        jump = None
        if op in ('checkflag', 'checkevent'):
            value = args[0] in flags
        elif op == 'readmem':
            require(args == [RUN], 'unexpected variable')
            value = wins
        elif op == 'yesorno':
            choices += 1
            value = yes
        elif op in ('ifequal', 'ifnotequal', 'ifgreater'):
            n = int(args[0])
            if {'ifequal': value == n, 'ifnotequal': value != n, 'ifgreater': value > n}[op]:
                jump = args[1]
        elif op in ('iftrue','iffalse'):
            if bool(value) == (op == 'iftrue'):
                jump = args[0]
        elif op == 'sjump':
            jump = args[0]
        elif op in ('jumptext', 'jumptextfaceplayer', 'writetext'):
            pages.append(args[0])
            if op != 'writetext':
                return pages, choices, writes, 'end'
        elif op in ('setevent','clearevent','loadmem','blackoutmod'):
            writes.append((op, *args))
        elif op in ('end','startbattle','warpfacing'):
            return pages, choices, writes, op
        else:
            require(op in ('faceplayer','opentext','closetext','promptbutton','waitbutton','winlosstext','loadtrainer'), f'unknown {op} in {name}')
        if jump:
            name, index = jump, 0
    raise AssertionError(f'nonterminating visit to {start}')


def verify(data):
    rooms = data['maps']
    lobby = rooms[LOBBY]
    badges = [r['args'][0] for r in lobby['scripts']['MnElyRegister'] if r['command']=='checkflag']
    require(len(set(badges)) == len(badges) == 8, 'eight independent badge checks')
    for badge_bits in product((False, True), repeat=8):
        flags = {b for b, bit in zip(badges, badge_bits) if bit}
        if len(flags) == 8:
            continue
        for restored in (False, True):
            first = badge_bits.index(False) + 1
            result = visit(lobby,'MnElyRegister',flags | ({CLEAR} if restored else set()))
            require(result == ([f'MnElyMissingBadge{first}Text'], 0, [], 'end'), f'badge gate {badge_bits}')
    for bits in product((False, True), repeat=len(CLUES)):
        flags = set(badges) | {k for (k,_), bit in zip(CLUES,bits) if bit}
        first = next((label for key,label in CLUES if key in flags), 'MnElyLookAtItasca')
        require(visit(lobby,'MnElyRegister',flags) == ([first+'Text'],0,[],'end'), f'wrong handoff {bits}')
    flags = {*badges, CLEAR}
    require(visit(lobby,'MnElyRegister',flags) == (['MnElyRegistrationText','MnElyPrepareOutsideText'],1,[],'end'), 'transactional registration refusal')
    _, menus, writes, exit_command = visit(lobby,'MnElyRegister',flags,yes=True)
    require(menus == 1 and writes == [('loadmem',RUN,'0'),('setevent',ACTIVE),('blackoutmod','MN_ELY_POKECENTER_1F')] and exit_command=='warpfacing', 'registered run')
    for i, name in enumerate(ROOMS):
        room = rooms[name]
        (battle,) = room['scripted_trainer_battles']
        ident = 'MN_CHAMPION' if i==4 else f'MN_LEAGUE_{i+1}'
        load = room['scripts'][battle['source_script']][battle['loadtrainer_command_index']]
        require(load['command']=='loadtrainer' and load['args'][1]==ident, f'{name} wrong trainer')
        for returning,active,wins,accept in product((False,True),(False,True),range(6),(False,True)):
            state = ({TITLE} if returning else set()) | ({ACTIVE} if active else set())
            pages, choices, writes, end = visit(room,name+'Challenge',state,wins,accept)
            choice = i==4 and returning and active and wins==i
            allowed = active and wins==i and (not choice or accept)
            require(choices == int(choice) and not writes and (end=='startbattle')==allowed, f'{name} {returning,active,wins,accept}')
            if choice and not accept:
                require(pages[-1]=='MnCaseyTakeABreathText', 'Casey refusal')
        sequence = room['scripts'][name+'BattleStart']
        ops = [r['command'] for r in sequence]
        require(ops.index('startbattle')<ops.index('reloadmapafterbattle')<ops.index('loadmem')<ops.index('changeblock')
                and sequence[ops.index('loadmem')]['args']==[RUN,str(i+1)], f'{name} progression')
    canonical=json.loads((REGION/'story.json').read_text())['league_return_dialogue']
    for n, labels in canonical.items():
        for label,pages in labels.items():
            r=rooms[n]
            require(native_pages(r['scripts'][label])==native_pages(r['script_text_bodies'][label]['commands'])==pages
                    and r['scripts'][label]==data['map_scripts'][label], f'unbound page {label}')
    print('510 missing-badge cases, 128 survey states, both registration decisions, and 240 five-room admission/return decisions passed. Declining Casey preserves the same four wins for a retry.')


if __name__ == '__main__':
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pack',type=Path)
    args=parser.parse_args()
    verify(load_pack(args.pack)['data'])
