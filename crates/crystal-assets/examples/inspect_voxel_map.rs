//! Print authored block coordinates for matching live geometry profiles.
//! Usage: inspect_voxel_map PACK MAP [MAP ...]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = std::env::args().skip(1);
    let path = args
        .next()
        .ok_or("usage: inspect_voxel_map PACK MAP [MAP ...]")?;
    let pack = crystal_assets::read_verified_compiled_game_pack(path)?;
    for name in args {
        let map = pack.data().map_module(&name)?;
        println!(
            "{name}: {}x{} blocks, {}",
            map.attributes.width, map.attributes.height, map.attributes.tileset_name
        );
        for (y, row) in map
            .blocks
            .chunks(usize::from(map.attributes.width))
            .enumerate()
        {
            print!("{y:02}: ");
            for block in row {
                print!("{block:02x} ");
            }
            println!();
        }
    }
    Ok(())
}
