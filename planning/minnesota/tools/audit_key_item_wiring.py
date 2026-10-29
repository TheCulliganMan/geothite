#!/usr/bin/env python3
"""Report Minnesota key-item script wiring without claiming earned access.

All script labels are scanned, including conditional and dormant programs.
Sources and declared field usability are leads for controller verification,
not proof of successful delivery, progression, persistence or reachable use.
"""
import argparse
import json

from render_reference import load_pack


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('pack')
    args = parser.parse_args()
    pack = load_pack(args.pack)
    data = pack['data']
    items = {name: item for name, item in data['items'].items()
             if item['pocket'] == 'KEY_ITEM'}
    references = {}
    grants = {'giveitem', 'verbosegiveitem', 'itemball', 'hiddenitem'}
    checks = {'checkitem'}
    consumes = {'takeitem'}
    for map_name, room in data['maps'].items():
        if not map_name.startswith('Mn'):
            continue
        for label, commands in room['scripts'].items():
            if not isinstance(commands, list):
                continue
            for index, command in enumerate(commands):
                operands = command.get('args', [])
                if not operands or operands[0] not in items:
                    continue
                opcode = command.get('command')
                # Text operands and incidental names are not item operations.
                if opcode not in grants | checks | consumes:
                    continue
                name = operands[0]
                entry = references.setdefault(name, dict(sources=[], checks=[], removals=[]))
                kind = 'sources' if opcode in grants else 'checks' if opcode in checks else 'removals'
                entry[kind].append(dict(map=map_name, script=label,
                                        command_index=index, command=opcode))
    report = {}
    for name, entry in sorted(references.items()):
        item = items[name]
        report[name] = dict(
            **entry,
            bag_description=item['description'],
            declared_field_usable=item['field_usable'],
            field_effect=item['effect'],
            has_script_source=bool(entry['sources']),
            has_script_check_or_removal=bool(entry['checks'] or entry['removals']))
    print(json.dumps(dict(
        pack=args.pack, content_identity=pack['identity']['content_hash'],
        referenced_regional_key_items=report,
        items_with_sources_but_no_script_use_or_declared_field_use=[
            name for name, item in report.items() if item['has_script_source']
            and not item['has_script_check_or_removal'] and not item['declared_field_usable']],
        caution=__doc__.strip()), indent=2))


if __name__ == '__main__':
    main()
