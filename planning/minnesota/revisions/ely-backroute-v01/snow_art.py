"""In-memory native pine import for source review; no pack or asset export."""
import copy,io,json
from PIL import Image

def snow_art(p):
 d=copy.deepcopy(p['data']['tilesets']['kanto']);f=p['runtime_files'];out={}
 for path,b in f.items():
  if path.startswith('gfx/tilesets/kanto.') or path.startswith('data/tilesets/kanto_') or path=='data/tilesets/kanto.json':out[path.replace('kanto','mn_ely_snow')]=b
 old=bytes(f['data/tilesets/kanto_metatiles.bin']);assert len(old)==131*16 and max(old)<96
 graphics=bytearray(f['gfx/tilesets/kanto.2bpp']);assert len(graphics)==96*16;graphics.extend(bytes(160*16))
 source=bytes(f['gfx/tilesets/mn_winter_north_shore.2bpp'])
 fg=json.loads(bytes(f['data/tilesets/mn_winter_north_shore_foregrounds.json']));newfg={}
 for target,original in zip(range(96,102),range(250,256)):
  graphics[target*16:(target+1)*16]=source[original*16:(original+1)*16];d['palette_map'][target]=0;newfg[str(target)]=fg[str(original)]
 recipe=[96,97,96,97,100,101,100,101,100,101,100,101,98,99,98,99]
 d['collision']['83']=['WALL']*4
 out['data/tilesets/mn_ely_snow_metatiles.bin']=old+bytes(recipe)
 out['data/tilesets/mn_ely_snow.json']=json.dumps(d).encode()
 out['data/tilesets/mn_ely_snow_palette_map.json']=json.dumps(d['palette_map']).encode()
 out['data/tilesets/mn_ely_snow_foregrounds.json']=json.dumps(newfg).encode()
 out['gfx/tilesets/mn_ely_snow.2bpp']=bytes(graphics)
 im=Image.new('L',(128,128));pixels=im.load()
 for y in range(128):
  for x in range(128):
   tile=(y//8)*16+x//8;row=tile*16+y%8*2;bit=7-x%8
   level=((graphics[row]>>bit)&1)|(((graphics[row+1]>>bit)&1)<<1);pixels[x,y]=[255,170,85,0][level]
 b=io.BytesIO();im.save(b,format='PNG');out['gfx/tilesets/mn_ely_snow.png']=b.getvalue()
 pal=bytes(f['gfx/tilesets/bg_tiles.pal']).decode().split('; day\n')[1].split('; nite')[0].strip().splitlines()[:8]
 pal[2]=' RGB 31,31,31, 27,29,31, 14,20,23, 07,07,07 ; snowy native green shapes'
 out['gfx/tilesets/mn_ely_snow.pal']=('\n'.join(pal)+'\n').encode()
 return d,out
