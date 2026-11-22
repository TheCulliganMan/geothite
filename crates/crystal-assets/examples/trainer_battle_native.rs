//! Encode native battle-only JSON without generating field sprites.
use std::fs;
use std::path::PathBuf;

use anyhow::{Result, ensure};

fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().skip(1).map(PathBuf::from).collect();
    let [source, output] = args.as_slice() else {
        anyhow::bail!("usage: trainer_battle_native SOURCE_JSON OUTPUT_ROOT");
    };
    let files = crystal_assets::encode_trainer_battle_native(&fs::read_to_string(source)?)?;
    ensure!(
        files.keys().all(|path| !output.join(path).exists()),
        "choose fresh native output paths"
    );
    for (path, bytes) in files {
        let path = output.join(path);
        fs::create_dir_all(path.parent().expect("native resource has parent"))?;
        use std::io::Write;
        fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path)?
            .write_all(&bytes)?;
    }
    println!("{} (three battle-only resources)", output.display());
    Ok(())
}
