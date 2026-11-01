#!/usr/bin/env python3
"""Fast, read-only source review of an explicit installed-city expansion.

No terrain is generated, no pack is written and no gameplay is simulated.
Every render stage is retained through the existing lossless render archive.
"""
import argparse
import hashlib
import json
from pathlib import Path
import time

from PIL import Image

from audit_city_expansions import module_hash, relocated
from render_archive import save_render
from render_reference import SourceArt, layout_blocks, load_pack, npc_palette_pack, room_sheet
from render_stages import render_stage
from review_city_native import priority_scene
from review_city_rooms_native import effective_period
from review_north_shore import oriented_sprite


FACING = {'UP':'north','DOWN':'south','LEFT':'west','RIGHT':'east'}


def sha(raw):
    return hashlib.sha256(raw).hexdigest()


def explicit_objects(source):
    """Source residents and optional trainers/items start with owned flags unset."""
    result = []
    for field in ['residents','trainers','items']:
        for actor in source.get(field,[]):
            result.append(dict(id=actor['id'],sprite=actor.get('sprite','SPRITE_POKE_BALL'),
                               tile=actor['tile'],facing=actor.get('facing','south'),
                               owned_flag=actor.get('flag',actor.get('defeated_flag'))))
    return result


def base_objects(module, contract):
    result = []
    for obj in relocated(module['objects'],'object',contract):
        direction = obj['spritemovedata'].rsplit('_',1)[-1]
        result.append(dict(id=obj.get('object_identifier') or obj['script'],sprite=obj['sprite'],
                           tile=[obj['x'],obj['y']],facing=FACING.get(direction,'south'),
                           original_movement=obj['spritemovedata'],owned_flag=obj.get('event_flag'),
                           native_palette_override=obj.get('pal',0)))
    return result


def collision_lookup(pack, attributes, blocks, recipes=()):
    collision = dict(pack['data']['tilesets'][attributes['tileset_name']]['collision'])
    for recipe in recipes:
        original = pack['data']['tilesets'][recipe['source_tileset']]['collision'][recipe['source_block']]
        assert recipe['collision'] == original, 'derived source collision changed'
        assert recipe.get('tile_row_order') is None and recipe.get('tile_row_sources') is None, 'native city block-row rearrangement is unsupported'
        collision[recipe['id']] = original
    width, height = attributes['width'],attributes['height']

    def kind(at):
        x,y = at
        assert 0 <= x < width*2 and 0 <= y < height*2, ('source core outside map',at)
        return collision[f'{blocks[y//2*width+x//2]:02x}'][y%2*2+x%2]
    return kind


def source_priority(pack, module, ground, scene, recipes=(), box=None):
    """Use the existing native mask helper on inherited and imported glyphs.

    Imported cells are grouped by their real native source tileset. Only an
    in-memory map view is adapted; no alias catalog or graphics are generated.
    Palette overrides preserve glyph indices and therefore the priority mask.
    """
    box = box or (0,0,scene.width,scene.height)
    if not recipes:
        return priority_scene(pack,module,scene,ground,box)
    by_id = {int(recipe['id'],16):recipe for recipe in recipes}
    # Block00 is not assumed to be safe in arbitrary source tilesets.
    def safe_block(tileset):
        return next(int(key,16) for key,tokens in pack['data']['tilesets'][tileset]['collision'].items()
                    if tokens in [['FLOOR']*4,['WALL']*4])
    safe = safe_block(module['attributes']['tileset_name'])
    base = dict(module,blocks=[safe if block in by_id else block for block in module['blocks']])
    masked = priority_scene(pack,base,scene,ground,box)
    for tileset in sorted({recipe['source_tileset'] for recipe in recipes}):
        safe = safe_block(tileset)
        attributes = dict(module['attributes'],tileset_name=tileset)
        blocks = [int(by_id[block]['source_block'],16) if block in by_id and by_id[block]['source_tileset'] == tileset else safe
                  for block in module['blocks']]
        masked += priority_scene(pack,dict(attributes=attributes,blocks=blocks),scene,ground,box)
    return masked


class ReviewArt:
    """Reuse source block, sprite and palette caches across a warm review loop."""
    def __init__(self, pack):
        self.pack = pack
        self.art = SourceArt(pack)
        self.sprite_packs = {}
        self.sprites = {}

    def sprite(self, name, facing, period):
        bank = 'day' if period == 'indoor' else period
        key = name,facing,bank
        if key not in self.sprites:
            if bank not in self.sprite_packs:
                self.sprite_packs[bank] = npc_palette_pack(self.pack,bank)
            self.sprites[key] = oriented_sprite(self.sprite_packs[bank],name,facing)
        return self.sprites[key]


def render_map(review, module, objects, period, directory, recipes=()):
    attr = module['attributes']
    ground = review.art.render(attr['tileset_name'],attr['width'],attr['height'],module['blocks'],period,derived=recipes)
    save_render(ground,directory/'whole-terrain-original.png')
    actor_layer = Image.new('RGBA',ground.size)
    for obj in objects:
        assert obj.get('native_palette_override',0) == 0, 'explicit nondefault NPC palette requires native review'
        frame = review.sprite(obj['sprite'],obj['facing'],period)
        save_render(frame,directory/'source-sprites'/f"{obj['sprite']}-{obj['facing']}.png")
        actor_layer.alpha_composite(frame,(obj['tile'][0]*16,obj['tile'][1]*16))
    save_render(actor_layer,directory/'whole-actors-rgba.png')
    scene = render_stage(ground,'convert',directory,'terrain-rgba','RGBA')
    scene.alpha_composite(actor_layer)
    save_render(scene,directory/'whole-before-native-priority.png')
    masked = source_priority(review.pack,module,ground,scene,recipes)
    save_render(scene,directory/'whole-source-scene.png')
    return ground,scene,masked


def player_camera(review,module,objects,ground,scene,period,view,directory,recipes=()):
    x,y = view['core']
    player = review.sprite('SPRITE_CHRIS','south',period)
    save_render(player,directory/'standing-player-rgba.png')
    with_player = scene.copy()
    with_player.alpha_composite(player,(x*16,y*16))
    save_render(with_player,directory/'whole-player-before-native-priority.png')
    masked = source_priority(review.pack,module,ground,with_player,recipes,
                             (x*16-96,y*16-88,x*16+96,y*16+88))
    save_render(with_player,directory/'whole-player-with-native-priority.png')
    # Extend small rooms with their declared native border without scaling them.
    border = review.art.blocks(module['attributes']['tileset_name'],period)[module['attributes']['border_block']]
    save_render(border,directory/'native-border-block.png')
    if with_player.width < 160 or with_player.height < 144:
        canvas = Image.new('RGBA',(max(160,with_player.width),max(144,with_player.height)))
        for yy in range(0,canvas.height,32):
            for xx in range(0,canvas.width,32):
                canvas.paste(border,(xx,yy))
        canvas.alpha_composite(with_player,(0,0))
        save_render(canvas,directory/'whole-native-border-padding.png')
    else:
        canvas = with_player
    left = max(0,min(canvas.width-160,x*16-80))
    top = max(0,min(canvas.height-144,y*16-72))
    frame = render_stage(canvas,'crop',directory,'source160',(left,top,left+160,top+144))
    save_render(frame,directory/'source160.png')
    enlarged = render_stage(frame,'resize',directory,'source4x',(640,576),Image.Resampling.NEAREST)
    save_render(enlarged,directory/'source4x.png')
    return frame,masked


def render_review(review, layout, contract, views, output, exterior_only=False):
    started = time.perf_counter()
    output.mkdir(parents=True,exist_ok=False)
    checks = []

    def check(label,valid,**details):
        checks.append(dict(check=label,passed=bool(valid),**details))

    pack, city = review.pack,contract['city']
    original = pack['data']['maps'][city]
    assert module_hash(original) == contract['native_module_sha256'], 'base city differs from explicit pre-expansion contract'
    assert layout['id'] == city and layout['dimensions_metatiles'] == contract['dimensions_metatiles']
    assert layout['tileset'] == original['attributes']['tileset_name']
    width,height = layout['dimensions_metatiles']
    attr = dict(original['attributes'],width=width,height=height)
    city_module = dict(id=city,attributes=attr,blocks=layout_blocks(layout))
    recipes = layout.get('derived_blocks',[])
    objects = base_objects(original,contract) + explicit_objects(contract)
    period = effective_period(attr)
    kind = collision_lookup(pack,attr,city_module['blocks'],recipes)
    for obj in objects:
        check(city+' actor native Floor '+obj['id'],kind(obj['tile']) == 'FLOOR',core=obj['tile'])
    for door in contract['doors']:
        label = door.get('id') or door['target']+' warp'+str(door['warp_index'])
        check(city+' paired facade '+label,kind(door['tile']) == 'DOOR' and kind(door['return_tile']) == 'FLOOR')
    ground,scene,masked = render_map(review,city_module,objects,period,output/'city',recipes)
    preview_size = (scene.width//2,scene.height//2)
    preview = render_stage(scene,'resize',output/'city','whole-half',preview_size,Image.Resampling.NEAREST)
    save_render(preview,output/'whole-review.png')
    modules = {city:(city_module,objects,period,ground,scene,recipes)}
    room_images, room_stats = [],[]
    for room in ([] if exterior_only else contract['rooms']):
        name = room['id']
        width,height = room['dimensions_metatiles']
        attrs = dict(pack['data']['maps'][room['template']]['attributes'],width=width,height=height,
                     tileset_name=room['tileset'],border_block=room['border_block'])
        module = dict(id=name,attributes=attrs,blocks=[int(value,16) for row in room['rows'] for value in row.split()])
        assert len(module['blocks']) == width*height, 'room row extent differs'
        room_kind = collision_lookup(pack,attrs,module['blocks'])
        bodies = explicit_objects(room)
        occupied = {tuple(obj['tile']) for obj in bodies}
        check(name+' safe native Floor arrival',room_kind(room['arrival_core']) == 'FLOOR' and tuple(room['arrival_core']) not in occupied)
        for obj in bodies:
            check(name+' actor native Floor '+obj['id'],room_kind(obj['tile']) == 'FLOOR',core=obj['tile'])
        for warp in room['warps']:
            token = room_kind(warp['tile'])
            expected = {'FLOOR'} if warp['passive'] else {'STAIRCASE','LADDER','WARP_CARPET_DOWN','WARP_CARPET_UP'}
            check(name+' warp source collision '+str(warp['index']),token in expected,core=warp['tile'],collision=token)
        group = effective_period(attrs)
        room_ground,room_scene,room_masked = render_map(review,module,bodies,group,output/'rooms'/name)
        room_images.append((name,room_scene))
        room_stats.append(dict(id=name,period=group,actors=len(bodies),foreground_pixels_over_actors=room_masked,
                               rows_sha256=sha(bytes(module['blocks']))))
        modules[name] = module,bodies,group,room_ground,room_scene,()
    for start in range(0,len(room_images),12):
        save_render(room_sheet(room_images[start:start+12],3),output/f'new-rooms-{start//12+1}.png')
    frames, camera_stats = [],[]
    requested = views or [dict(id='city-arrival',map=city,core=contract['arrival_core'])]
    for index, view in enumerate(requested):
        name = view['map']
        assert name in modules, 'camera must belong to the rendered city/new rooms; --exterior-only accepts city views only'
        module,bodies,group,ground,scene,recipes = modules[name]
        label = view.get('id',f'{index:02}-{name}')
        assert Path(label).name == label and label not in {'','.','..'}, 'camera ID must be a plain filename'
        point = tuple(view['core'])
        token = collision_lookup(pack,module['attributes'],module['blocks'],recipes)(point)
        check(name+' camera unoccupied native Floor '+label,token == 'FLOOR' and all(tuple(obj['tile']) != point for obj in bodies),core=point)
        frame,camera_masked = player_camera(review,module,bodies,ground,scene,group,view,output/'cameras'/f'{index:02}-{label}',recipes)
        frames.append((name+' '+label,frame))
        camera_stats.append(dict(id=label,map=name,core=point,effective_period=group,
                                 source_rgba_sha256=sha(frame.tobytes()),foreground_pixels_over_player=camera_masked))
    save_render(room_sheet(frames,3),output/'camera-review.png')
    source_pixels = []
    for recipe in layout.get('derived_blocks',[]):
        block = review.art.blocks(recipe['source_tileset'],period,recipe.get('palette_override'))[int(recipe['source_block'],16)]
        save_render(block,output/'source-blocks'/f"{recipe['id']}-from-{recipe['source_tileset']}-{recipe['source_block']}.png")
        source_pixels.append(dict(id=recipe['id'],source_tileset=recipe['source_tileset'],source_block=recipe['source_block'],
                                  rgba_or_rgb_sha256=sha(block.tobytes()),collision=recipe['collision']))
    return dict(status='SOURCE REVIEW ONLY',static_checks='PASS' if all(row['passed'] for row in checks) else 'REVIEW_ERRORS',
                city=city,dimensions_metatiles=layout['dimensions_metatiles'],rooms=len(room_stats),
                review_scope='EXTERIOR ONLY; new rooms and room checks skipped' if exterior_only else 'Whole exterior and all new contract rooms',
                object_visibility='All source-authored objects; new pickup/trainer flags initially unset. Scripts and progression are not simulated.',
                native_masks='Existing priority_scene applied to original glyphs and exact native imported-block source masks.',
                native_publication_required=True,controller_or_gpu_evidence=False,checks=checks,
                errors=[row for row in checks if not row['passed']],room_stats=room_stats,cameras=camera_stats,
                exterior_effective_period=period,exterior_foreground_pixels_over_actors=masked,source_block_pixels=source_pixels,
                render_elapsed_seconds=round(time.perf_counter()-started,6))


def watch_reviews(args, review, pack_sha256, load_elapsed):
    """Reload edited JSON inputs while retaining one loaded pack/art cache.

    Each attempted revision retains its captured bytes before parsing/rendering.
    A change during rendering is reported and causes a subsequent revision.
    """
    roles = {'layout':args.layout,'contract':args.contract}
    if args.views:
        roles['views'] = args.views
    tool_bytes = Path(__file__).read_bytes()
    last_hashes = None
    revisions = []
    interrupted = False
    pack_stat = args.base.stat()
    pack_signature = pack_stat.st_dev,pack_stat.st_ino,pack_stat.st_size,pack_stat.st_mtime_ns

    def capture():
        return {role:path.read_bytes() for role,path in roles.items()}

    try:
        while args.max_renders is None or len(revisions) < args.max_renders:
            try:
                captured = capture()
            except OSError:
                # Editors may briefly replace/delete a file during an atomic save.
                time.sleep(0.25)
                continue
            hashes = {str(roles[role]):sha(raw) for role,raw in captured.items()}
            if hashes == last_hashes:
                time.sleep(0.25)
                continue
            # Avoid capturing an ordinary partial multi-write save as a revision.
            time.sleep(0.25)
            try:
                if captured != capture():
                    continue
            except OSError:
                continue
            current_stat = args.base.stat()
            current_signature = current_stat.st_dev,current_stat.st_ino,current_stat.st_size,current_stat.st_mtime_ns
            if current_signature != pack_signature:
                assert sha(args.base.read_bytes()) == pack_sha256, 'base pack changed; restart watch to reload native assets'
                pack_signature = current_signature
            last_hashes = hashes
            digest = sha(json.dumps(hashes,sort_keys=True).encode())
            directory = args.output/f'revision-{len(revisions):03}-{digest[:12]}'
            directory.mkdir(exist_ok=False)
            snapshots = directory/'inputs'
            snapshots.mkdir()
            for role,raw in captured.items():
                (snapshots/(role+'.json')).write_bytes(raw)
            (directory/'review_major_city.py.snapshot').write_bytes(tool_bytes)
            result = None
            error = None
            started = time.perf_counter()
            try:
                layout,contract = json.loads(captured['layout']),json.loads(captured['contract'])
                views = json.loads(captured['views']) if 'views' in captured else None
                if views is not None:
                    assert isinstance(views,list) and views, 'explicit camera array required'
                result = render_review(review,layout,contract,views,directory/'render-00',args.exterior_only)
                result['warm_cache'] = bool(revisions)
                result['output'] = str(directory/'render-00')
                (directory/'render-00'/'source-review.json').write_text(json.dumps(result,indent=2)+'\n')
            except Exception as failure:
                error = dict(type=type(failure).__name__,message=str(failure))
            try:
                unchanged = captured == capture()
            except OSError:
                unchanged = False
            receipt = dict(status='SOURCE REVIEW ONLY' if error is None else 'SOURCE REVIEW ERROR',
                           watch_revision=len(revisions),input_sha256={str(args.base):pack_sha256,**hashes},
                           inputs_unchanged=unchanged,tool_sha256=sha(tool_bytes),
                           pack_load_seconds=round(load_elapsed,6) if not revisions else 0,
                           reused_pack_and_art_cache=bool(revisions),exterior_only=args.exterior_only,
                           revision_elapsed_seconds=round(time.perf_counter()-started,6),
                           runs=[result] if result else [],error=error,
                           snapshot_policy='Rendered captured input bytes; changes during rendering require the next immutable revision.',
                           native_publication_required=True,controller_or_gpu_evidence=False)
            raw = (json.dumps(receipt,indent=2)+'\n').encode()
            receipt_path = directory/('receipt-'+sha(raw)+'.json')
            receipt_path.write_bytes(raw)
            revisions.append(dict(receipt=str(receipt_path),revision=len(revisions),
                                  inputs_unchanged=unchanged,error=error,
                                  static_checks=result['static_checks'] if result else 'ERROR',
                                  render_seconds=result['render_elapsed_seconds'] if result else None))
            print(json.dumps(revisions[-1]),flush=True)
    except KeyboardInterrupt:
        interrupted = True
    summary = dict(status='SOURCE REVIEW ONLY',mode='watch',revisions=revisions,
                   pack_load_seconds=round(load_elapsed,6),interrupted=interrupted,
                   max_renders=args.max_renders,tool_sha256=sha(tool_bytes),
                   native_publication_required=True,controller_or_gpu_evidence=False)
    raw = (json.dumps(summary,indent=2)+'\n').encode()
    target = args.output/('watch-receipt-'+sha(raw)+'.json')
    target.write_bytes(raw)
    print(json.dumps(dict(watch_receipt=str(target),revisions=len(revisions)),indent=2),flush=True)
    return 0 if revisions and revisions[-1]['error'] is None and revisions[-1]['inputs_unchanged'] and revisions[-1]['static_checks'] == 'PASS' else 1


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    for name in ['base','layout','contract','output']:
        parser.add_argument('--'+name,type=Path,required=True)
    parser.add_argument('--views',type=Path,help='JSON array of {id,map,core}; city or new room names')
    parser.add_argument('--exterior-only',action='store_true',help='Street review only: skip all new room renders and room checks')
    parser.add_argument('--warm-rerenders',type=int,default=0,choices=range(0,4),
                        help='Retain additional reviews in this process, reusing loaded pack/block/sprite caches')
    parser.add_argument('--watch',action='store_true',help='Reload edited layout/contract/views, retaining cached native pack/art and immutable revisions')
    parser.add_argument('--max-renders',type=int,help='Stop watch after this many attempted input revisions (otherwise Ctrl-C)')
    args = parser.parse_args()
    if args.max_renders is not None and (not args.watch or args.max_renders < 1):
        parser.error('--max-renders requires --watch and a positive count')
    if args.watch and args.warm_rerenders:
        parser.error('--watch reloads changed inputs; use --warm-rerenders separately for fixed-input benchmarks')
    args.output.mkdir(parents=True,exist_ok=False)
    paths = [args.base,args.layout,args.contract] + ([args.views] if args.views else [])
    source_bytes = {path:path.read_bytes() for path in paths if path != args.base}
    hashes = {str(path):sha(source_bytes[path] if path in source_bytes else path.read_bytes()) for path in paths}
    started = time.perf_counter()
    pack = load_pack(args.base)
    load_elapsed = time.perf_counter()-started
    if args.watch:
        raise SystemExit(watch_reviews(args,ReviewArt(pack),hashes[str(args.base)],load_elapsed))
    layout,contract = json.loads(source_bytes[args.layout]),json.loads(source_bytes[args.contract])
    views = json.loads(source_bytes[args.views]) if args.views else None
    if views is not None:
        assert isinstance(views,list) and views, 'explicit camera array required'
    review = ReviewArt(pack)
    runs = []
    for index in range(args.warm_rerenders+1):
        directory = args.output/f'render-{index:02}'
        result = render_review(review,layout,contract,views,directory,args.exterior_only)
        result['warm_cache'] = index > 0
        result['output'] = str(directory)
        runs.append(result)
        (directory/'source-review.json').write_text(json.dumps(result,indent=2)+'\n')
        print(contract['city'],'warm' if index else 'initial','render_seconds',result['render_elapsed_seconds'],
              result['static_checks'],flush=True)
    snapshots = args.output/'inputs'
    snapshots.mkdir()
    for role,path in [('layout',args.layout),('contract',args.contract),('views',args.views)]:
        if path:
            (snapshots/(role+'.json')).write_bytes(source_bytes[path])
    unchanged = hashes == {str(path):sha(path.read_bytes()) for path in paths}
    receipt = dict(status='SOURCE REVIEW ONLY',input_sha256=hashes,inputs_unchanged=unchanged,
                   tool_sha256=sha(Path(__file__).read_bytes()),pack_load_seconds=round(load_elapsed,6),
                   exterior_only=args.exterior_only,
                   timing_scope='Each render includes layout rasterization, source checks, native art/priority, PNG/archive writes and all room/camera sheets; pack load measured separately.',
                   runs=runs,native_publication_required=True,controller_or_gpu_evidence=False)
    raw=(json.dumps(receipt,indent=2)+'\n').encode()
    target=args.output/('receipt-'+sha(raw)+'.json')
    target.write_bytes(raw)
    (args.output/'review_major_city.py.snapshot').write_bytes(Path(__file__).read_bytes())
    print(json.dumps(dict(receipt=str(target),pack_load_seconds=receipt['pack_load_seconds'],
                          render_seconds=[row['render_elapsed_seconds'] for row in runs],
                          errors=[error for row in runs for error in row['errors']]),indent=2))
    raise SystemExit(0 if unchanged and all(not row['errors'] for row in runs) else 1)


if __name__ == '__main__':
    main()
