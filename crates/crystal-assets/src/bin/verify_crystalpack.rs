use anyhow::{Context,Result,ensure};
use crystal_assets::{read_verified_compiled_game_pack,ModpackAudioKind};
use crystal_audio::synth::{decode_midi,render,SynthContext};

fn main()->Result<()> {
    let mut args=std::env::args().skip(1);
    let path=args.next().context("usage: verify_crystalpack <pack> [--audio]")?;
    let audio=match args.next().as_deref() {None=>false,Some("--audio")=>true,_=>anyhow::bail!("unknown audit option")};
    ensure!(args.next().is_none(),"unexpected audit argument");
    let pack=read_verified_compiled_game_pack(&path).context("verify complete game pack")?;
    if audio {
        for asset in &pack.data().audio {
            let program=decode_midi(&asset.midi_program.as_ref().with_context(||format!("{} has no source audio program",asset.id))?.midi_base64)?;
            let rendered=render(&program,SynthContext::cartridge()?).with_context(||format!("audit synthesis {}",asset.id))?;
            let pcm=rendered.downsample();let frames=pcm.len()/2;
            let hash=pcm.iter().flat_map(|sample|sample.to_le_bytes()).fold(2166136261u32,|hash,byte|(hash^u32::from(byte)).wrapping_mul(16777619));
            let hash=format!("{hash:08x}");
            ensure!(asset.payload_hash.as_ref()==Some(&hash),"{} PCM hash mismatch",asset.id);
            ensure!(asset.pcm_frame_count==Some(frames),"{} frame count mismatch",asset.id);
            let primary=program.music_data.channels.values().filter_map(|source|source.number).min().context("missing audio channel")?;
            let start=if matches!(asset.kind,ModpackAudioKind::Music) {rendered.loop_samples.get(&primary).map(|sample|sample.div_ceil(2))}else{None};
            ensure!(asset.loop_start_sample==start && asset.loop_end_sample==start.map(|_|frames),"{} loop range mismatch",asset.id);
        }
    }
    println!("verified {path}: {} maps, {} species, {} moves, {} audio programs{}",pack.data().maps.len(),pack.data().pokemon.len(),pack.data().moves.len(),pack.data().audio.len(),if audio {"; all PCM hashes, frame counts and loop ranges match"}else{""});
    Ok(())
}
