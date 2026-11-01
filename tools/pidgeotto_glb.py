#!/usr/bin/env python3
"""Canonical Pidgeotto standard skin with bounded source-equivalent neutral reads."""
import argparse
from pathlib import Path
FILE='pidgeotto.glb';MAX_BYTES=1024*1024;PART_COUNT=34;CLIP_NAME='pidgeotto.idle'
def export_pidgeotto(model):
 from pidgeotto_skin_glb import export_pidgeotto_skin
 return export_pidgeotto_skin('pidgeotto',model)
def decode_pidgeotto(blob):
 from pidgeotto_skin_glb import decode_pidgeotto_skin
 return decode_pidgeotto_skin(blob)
def read_pidgeotto(path):
 with Path(path).open('rb') as f:return decode_pidgeotto(f.read(MAX_BYTES+1))
def load_pidgeotto(directory):
 source=Path(directory)/'pidgeotto.mesh.json'
 if source.exists():
  from model_asset_storage import read_model_json
  return read_model_json(source)
 return read_pidgeotto(Path(directory)/FILE)
def species_paths(directory):
 from battle_model_assets import species_paths as canonical_paths
 return canonical_paths(directory)
def main():
 from model_asset_storage import read_model_json
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('source',type=Path);p.add_argument('output',type=Path);a=p.parse_args()
 blob=export_pidgeotto(read_model_json(a.source));a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_bytes(blob)
 print(f'Wrote {a.output}: {len(blob):,} bytes; 34 anatomy parts; 14-control standard skin')
if __name__=='__main__':main()
