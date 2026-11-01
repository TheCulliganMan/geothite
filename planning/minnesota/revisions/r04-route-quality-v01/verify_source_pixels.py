"""Verify exact old art and row donor pixels across all four native time banks."""
from pathlib import Path
from copy import deepcopy
import sys,json,cbor2,hashlib
from PIL import Image,ImageChops
ROOT=Path(__file__).resolve().parents[4];sys.path.insert(0,str(ROOT/'planning/minnesota/tools'))
from render_reference import SourceArt,room_sheet
from render_archive import save_render
p=ROOT/'content-packs/minnesota-river-long-v02.crystalpack';raw=p.read_bytes();sha=lambda b:hashlib.sha256(b).hexdigest()
assert sha(raw)=='3ddbad16c6538e52f219c12497e4fbe6e41a7f18af5c42589b7b92357cfad243'
base=cbor2.loads(raw[22:]);folder=ROOT/'planning/minnesota/revisions/r04-route-quality-v01/source-v05';contract=json.loads((folder/'native.contract.json').read_bytes())
preview=deepcopy(base);ts='mn_river_park';path=f'data/tilesets/{ts}_metatiles.bin';old=bytes(base['runtime_files'][path]);new=old+b''.join(bytes(b['glyphs'])for b in contract['blocks']);preview['runtime_files'][path]=new
out=ROOT/'planning/minnesota/renders/r04-route-quality-v01/combined-source-v05-art-proof-v01';out.mkdir(exist_ok=False)
results=[];frames=[]
for bank in ['morn','day','nite','dark']:
 a=SourceArt(base).blocks(ts,bank);b=SourceArt(preview).blocks(ts,bank)
 assert len(a)==68 and len(b)==72
 for before,after in zip(a,b):assert ImageChops.difference(before,after).getbbox()is None
 for block in contract['blocks']:
  expected=Image.new('RGB',(32,32))
  for row,source in enumerate(block['row_sources']):expected.paste(a[source['block']].crop((0,source['row']*8,32,source['row']*8+8)),(0,row*8))
  assert ImageChops.difference(expected,b[block['target']]).getbbox()is None
  frames.append((bank+' / '+f"{block['target']:02x}",b[block['target']]))
 results.append(dict(bank=bank,old_blocks=68,old_pixels_compared=68*1024,new_blocks=4,new_pixels_compared=4*1024,old_pixel_differences=0,new_donor_row_pixel_differences=0))
save_render(room_sheet(frames,4),out/'four-bank-native-row-recipes.png')
proof=dict(status='SOURCE_PIXEL_PROOF_NOT_GPU',parent_pack_sha256=sha(raw),native_contract_sha256=sha((folder/'native.contract.json').read_bytes()),old_raw_prefix_exact=new[:len(old)]==old,old_glyph_files_palettes_and_collision_untouched=True,old_art_pixels_compared=68*1024*4,new_recipe_pixels_compared=4*1024*4,total_pixel_differences=0,banks=results,pack_written=False)
(out/'proof.json').write_text(json.dumps(proof,indent=2)+'\n');print(json.dumps(proof))
