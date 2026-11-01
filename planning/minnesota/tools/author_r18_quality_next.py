#!/usr/bin/env python3
"""Explicit R18 whole-trip authoring. Writes candidate sources only, never packs."""
import argparse
import json
from copy import deepcopy
from pathlib import Path
ROOT=Path(__file__).resolve().parents[3];BASE=ROOT/'planning/minnesota'

def main():
 p=argparse.ArgumentParser();p.add_argument('--revision',default='v01');a=p.parse_args();out=BASE/'renders/route-quality-next/r18'/a.revision;out.mkdir(parents=True,exist_ok=False)
 source=BASE/'routes';old=json.loads((source/'MN_R18.json').read_text());d=json.loads((source/'MN_R18.ribbon.candidate.json').read_text());l=json.loads((source/'MN_R18.ribbon.layout.json').read_text())
 # A fitted northern headland, not a floating tree maze. It joins the existing
 # grove and moves the trip below the meadow to a short protected lake frame.
 l['layers'] += [
  dict(kind='block_rect',name='Brainerd approach attached north wood shoulder',bounds=[4,3,6,5],block='05'),
  dict(kind='block_rect',name='Brainerd marina fork attached western wood shoulder',bounds=[4,7,5,9],block='05'),
  dict(kind='block_rect',name='Marina registration approach fitted western wood shoulder',bounds=[4,10,5,13],block='05'),
  dict(kind='block_rect',name='Western meadow headland rooted to existing north grove',bounds=[10,4,11,7],block='05'),
  dict(kind='block_rect',name='Western meadow headland fitted lower east shoulder',bounds=[11,5,12,7],block='05'),
  dict(kind='point',name='Western meadow headland tapered last shoreline shoulder',position=[12,6],block='05'),
  dict(kind='path',name='Western trip bends to actual lake before the meadow report fork',material='floor',points=[[6,6],[9,6],[9,7],[13,7],[13,6],[15,6]],width_metatiles=1),
  dict(kind='block_rect',name='Western lake bend protected native south bank',bounds=[10,7,16,8],block='49'),
  dict(kind='point',name='Western lake bend landward protected bank joint',position=[9,7],block='49'),
  dict(kind='point',name='Western lake bend fitted southeast bank cap',position=[16,7],block='4a'),
  dict(kind='point',name='Optional offshore islet native pine crown',position=[29,16],block='05'),
  dict(kind='point',name='Optional offshore islet flower verge beside retained reward',position=[30,16],block='04'),
 ]
 d['main_path_metatiles']=[[1,5],[6,5],[6,6],[9,6],[9,7],[13,7],[13,6],[22,6],[22,8],[24,8],[24,5],[38,5]]
 d['viewpoints'] += [dict(id='WEST_MARINA_FORK',tile=[11,11]),dict(id='WEST_MEADOW_BANK_TURN',tile=[19,14]),dict(id='WEST_LAKE_BEND',tile=[27,14]),dict(id='MEADOW_RETURN_FORK',tile=[27,12])]
 d['surf_viewpoints'] += [dict(id='ISLET_PINE_DISCOVERY',tile=[60,34])]
 d['review_contract']['whole_trip_next']=dict(status='SOURCE_CANDIDATE_ONLY; current660 native preimage required',western_main_course_core=[[12,12],[18,12],[18,14],[26,14],[26,12],[30,12]],meaning='Brainerd fork / western lake-facing bend / meadow report / required habitat / dune lookout or grassy shortcut / public launch / protected shore loop / beach',native_gameplay_proved=False)
 for n,v in [('MN_R18.candidate.json',d),('MN_R18.layout.candidate.json',l)]: (out/n).write_text(json.dumps(v,indent=2)+'\n')
 for n in ['MN_R18.json','MN_R18.layout.json']:(out/('before-'+n)).write_bytes((source/n).read_bytes())
 for n in ['MN_R18.shorewalk.candidate.json','MN_R18.shorewalk.layout.json','MN_R18.ribbon.candidate.json','MN_R18.ribbon.layout.json']:(out/('prior-'+n)).write_bytes((source/n).read_bytes())
 (out/'author_r18_quality_next.py.snapshot').write_bytes(Path(__file__).read_bytes());print(out)
if __name__=='__main__':main()
