"""Author the promoted/restored families' voices; retain retired scores in one book.

Scores are original creature-specific performances. Synthesis/export stays in Rust.
"""
import copy
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1] / 'pokemon'


def pulse(notes, duty=(2, 2, 2, 2), delay=0):
    return dict(delay=delay, duty=list(duty), notes=[list(n) for n in notes], duties={}, contours={})


def score(main, answer=None, breath=None):
    return dict(pulse1=main, pulse2=answer, noise=breath)


def noise(delay, notes):
    return dict(delay=delay, notes=[list(n) for n in notes])


# Independent second voices answer and interrupt rather than universally doubling.
# Register variants are explicit family decisions, never name hashes or ID offsets.
SPECS = {
'MN_F146': ('Sheltering snails: a soft questioning lip-pop settles into a hollow mantle hum.',
 score(pulse([(4,950,10,2),(3,0,0,0),(12,710,8,3)],(0,0,2,0)),pulse([(5,1470,5,2),(8,1190,4,3)],(0,0,0,0),5),noise(14,[(3,34,3,2)])),
 [(35,208,[]),(-10,256,[(7,820,7,2)]),(-45,288,[(4,0,0,0),(12,570,9,3)])]),
'MN_F147': ('Snowshoe hares: a quick alert chirrup spills into a thin breathy rising whistle.',
 score(pulse([(2,2270,11,1),(3,1920,9,1),(4,0,0,0),(8,2580,8,2)],(0,0,0,0)),pulse([(3,1270,5,2),(6,1730,4,2)],(1,0,1,0),7),noise(3,[(4,17,3,1),(5,37,2,2)])),
 [(10,224,[]),(38,256,[(5,2790,7,2)])]),
'MN_F148': ('Forest-gap owls: separated rounded hoots leave a dark hollow pause; an upper throat answers late.',
 score(pulse([(7,660,11,2),(5,0,0,0),(10,810,9,3)],(2,2,0,2)),pulse([(4,1310,5,1),(4,0,0,0),(8,1090,6,2)],(0,0,0,0),6),noise(12,[(5,66,3,2)])),
 [(18,224,[]),(-12,264,[(6,750,8,2)]),(-35,280,[(6,0,0,0),(12,560,10,3)])]),
'MN_F149': ('Deceptive berry shrub: a sweet piercing lure peep breaks into a lower sour gurgling chuckle.',
 score(pulse([(4,2410,11,1),(5,2050,9,2),(2,0,0,0),(3,780,11,1),(3,940,10,1),(9,610,8,3)],(0,2,0,3)),pulse([(4,420,6,2),(7,530,5,3)],(2,2,2,2),11),noise(15,[(3,52,5,1),(6,85,3,2)])),
 [(0,256,[])]),
'MN_F150': ('Basalt bears: a gruff double bark presses into a coarse resonant shoulder growl.',
 score(pulse([(5,510,12,1),(2,0,0,0),(7,390,11,2),(9,440,8,3)],(2,3,2,3)),pulse([(6,270,6,2),(11,310,5,3)],(1,1,1,1),5),noise(0,[(3,83,6,1),(10,102,4,3)])),
 [(15,240,[]),(-45,296,[(4,0,0,0),(13,350,10,3)])]),
'MN_F151': ('Wayfinding wolves: an inquisitive upward call gets a distant lower answer, then a bright locating tag.',
 score(pulse([(5,1120,11,2),(10,1490,10,3),(4,0,0,0),(6,1280,8,2)],(1,1,2,1)),pulse([(7,760,6,2),(10,920,5,3)],(0,0,0,0),7),noise(2,[(5,51,3,2)])),
 [(32,224,[]),(0,256,[(6,1670,7,2)]),(-18,280,[(5,0,0,0),(11,1810,9,3)])]),
'MN_F152': ('Hibernating bats: a hushed two-part trill hangs in a deliberate silent gap before a sudden released chirp.',
 score(pulse([(2,2790,9,1),(3,2320,8,1),(7,0,0,0),(3,3140,11,1),(7,2610,7,2)],(0,0,0,0)),pulse([(5,1430,4,2),(5,0,0,0),(5,1890,5,2)],(0,1,0,1),4),noise(13,[(3,9,2,1)])),
 [(25,216,[]),(40,248,[(4,2880,7,2)]),(-20,272,[(4,0,0,0),(10,2240,9,3)])]),
'MN_F153': ('Bank swallows: dry beak ticks lift into a narrow clear whistle with a separately timed throat catch.',
 score(pulse([(2,1840,11,1),(2,0,0,0),(2,2110,10,1),(4,0,0,0),(9,2490,8,2)],(0,1,0,0)),pulse([(3,1220,5,1),(6,1530,4,2)],(0,0,0,0),8),noise(0,[(2,18,4,1)])),
 [(15,224,[]),(-8,256,[(4,2250,8,2)])]),
'MN_F154': ('Snow buntings: two unequal fluted bursts curl upward, then a quick soft flock-answer flicks away.',
 score(pulse([(3,2140,10,1),(5,2770,9,2),(3,0,0,0),(2,2480,10,1),(6,3010,7,2)],(0,0,1,0)),pulse([(3,1530,5,2),(5,1840,4,3)],(0,0,0,0),9),noise(5,[(2,33,2,1)])),
 [(12,224,[]),(28,240,[(4,2640,7,2)]),(0,264,[(2,0,0,0),(7,3190,8,2)])]),
'MN_F155': ('Ice-sailing dragonflies: a taut papery buzzing chirr alternates with short hard foot-clicks.',
 score(pulse([(3,1710,10,1),(4,2040,9,1),(3,1660,10,1),(6,2230,7,2)],(0,3,0,3)),pulse([(5,840,5,2),(7,1170,4,3)],(1,0,1,0),4),noise(1,[(3,25,5,1),(3,41,3,1),(5,57,4,2)])),
 [(22,224,[]),(-20,272,[(4,0,0,0),(9,1860,9,2)])]),
'MN_F156': ('Tactunnel: dry nasal contact clicks interrupt a soft underground whuffle and a questioning little squeak.',
 score(pulse([(2,1460,10,1),(3,990,9,1),(3,0,0,0),(5,730,11,2),(7,1210,8,2)],(1,2,1,0)),pulse([(4,430,5,2),(8,570,4,3)],(2,2,2,2),6),noise(0,[(2,35,4,1),(4,68,3,2)])),
 [(0,256,[])]),
'MN_F157': ('Regenerating prairie skinks: an elastic bright chirr recoils through a quiet gap into a separately rising tail-answer.',
 score(pulse([(4,1610,11,1),(6,2070,9,2),(3,0,0,0),(7,1440,8,2)],(0,1,2,0)),pulse([(3,2710,5,1),(6,2340,4,2)],(0,0,0,0),9),noise(4,[(3,24,3,1)])),
 [(18,232,[]),(-12,264,[(5,1960,9,2),(4,2430,7,2)])]),
'MN_F158': ('Pendulum jumping mouse: one clear high peep swings into an off-center lower note, then springs upward again.',
 score(pulse([(4,2890,11,1),(7,1720,9,2),(3,0,0,0),(8,2630,8,3)],(0,0,0,0)),pulse([(4,1310,5,2),(8,1570,4,2)],(1,1,0,1),5),noise(14,[(2,16,2,1)])),
 [(0,248,[])]),
'MN_F159': ('Earth-shear badgers: an uneven pair of closed-mouth grunts opens into a rough assertive digging bark.',
 score(pulse([(4,710,11,1),(2,0,0,0),(6,540,12,2),(8,610,9,3)],(2,3,2,2)),pulse([(5,360,6,2),(9,440,5,3)],(1,1,2,1),4),noise(1,[(3,69,5,1),(8,85,4,3)])),
 [(28,224,[]),(0,256,[(5,810,9,2)]),(-32,288,[(3,0,0,0),(11,470,11,3)])]),
'MN_F013': ('Aurora creatures: a small luminous questioning whistle splits into unequal upper and lower curtain voices.',
 score(pulse([(4,2510,10,2),(7,2980,8,3),(5,2210,7,2)],(0,1,0,2)),pulse([(4,1670,5,2),(8,2030,4,3)],(0,0,0,0),6),noise(10,[(3,17,2,2)])),
 [(10,224,[]),(24,256,[(7,3230,7,2)]),(-12,280,[(4,0,0,0),(12,2710,8,3)])]),
'MN_F035': ('Berry muffin imps: a round cheek peep develops into a mischievous two-syllable chuckle with a soft crumbly throat catch.',
 score(pulse([(4,1870,11,1),(4,1530,9,2),(5,2130,8,2)],(0,2,0,1)),pulse([(4,840,5,2),(6,1090,4,3)],(2,2,0,2),6),noise(3,[(3,52,3,1)])),
 [(20,224,[]),(-18,272,[(3,0,0,0),(5,1330,10,1),(7,1770,7,3)])]),
'MN_F080': ('Mirage tricksters: a low voice rises while a delayed upper voice falls, seeming to exchange places mid-call.',
 score(pulse([(5,930,10,2),(7,1460,9,2),(8,2110,7,3)],(1,0,1,0)),pulse([(5,2640,6,2),(7,1930,5,2),(8,1210,4,3)],(0,0,0,0),3),noise(12,[(3,33,2,2)])),
 [(24,232,[]),(-24,272,[(4,0,0,0),(8,1690,8,2)])]),
'MN_F088': ('Water-power runners: skipping wet chirps grow into a strong pulsing throat call with a revolving lower answer.',
 score(pulse([(3,1540,11,1),(3,1980,9,1),(2,0,0,0),(5,1710,10,2),(7,2250,7,3)],(0,2,1,2)),pulse([(4,760,6,2),(6,1080,5,2),(5,890,4,3)],(1,2,1,0),5),noise(2,[(3,40,4,1),(6,57,3,2)])),
 [(20,224,[]),(-8,256,[(5,1820,9,2)]),(-28,288,[(3,0,0,0),(11,1310,10,3)])]),
'MN_F132': ('Snapping turtles: a dry jaw-click precedes a thick rasp and a low rolling throat note.',
 score(pulse([(3,1210,11,1),(5,610,12,2),(9,480,9,3)],(2,3,2,3)),pulse([(5,290,6,2),(9,350,5,3)],(1,1,1,1),3),noise(0,[(2,51,6,1),(7,87,4,3)])),
 [(14,240,[]),(-38,296,[(3,0,0,0),(12,390,10,3)])]),
}


def main():
    roster = json.loads((ROOT/'roster.json').read_text())
    curation = json.loads((ROOT/'roster-curation.json').read_text())
    retired = {f['id'] for f in curation['retired_families']}
    book = json.loads((ROOT/'cry-designs.json').read_text())
    archived = book.setdefault('archived_families', {})
    # Move, never duplicate, inactive authored programs into the same source book.
    for fid in list(book['families']):
        if fid in retired:
            archived[fid] = book['families'].pop(fid)
    for f in roster['families']:
        fid = f['id']
        if fid not in SPECS:
            continue
        intent, base, variants = SPECS[fid]
        assert len(variants) == len(f['members'])
        species = {}
        for member, (pitch, length, tail) in zip(f['members'], variants):
            variant = dict(pitch=pitch, length=length,
                           character=member['name']+': '+intent)
            if tail:
                custom = copy.deepcopy(base)
                custom['pulse1']['notes'].extend(copy.deepcopy(tail))
                variant['score'] = custom
            species[member['id']] = variant
        book['families'][fid] = dict(intent=intent, score=base, species=species)
        archived.pop(fid, None)
    active = {f['id'] for f in roster['families'] if f['id'] not in retired}
    assert set(book['families']) == active
    assert sum(len(f['species']) for f in book['families'].values()) == 225
    (ROOT/'cry-designs.json').write_text(json.dumps(book,indent=2)+'\n')
    print('Authored43 new/restored voices across19families; exact225active coverage; retired scores retained in same source book.')


if __name__ == '__main__':
    main()
