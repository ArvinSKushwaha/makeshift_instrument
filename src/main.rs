use std::{time::Duration, f32::consts::TAU};

use makeshift_instrument::{engine::Engine, soundfont2::SoundFont};

fn main() -> anyhow::Result<()> {
    let path = "/home/creme_brulee/Downloads/FluidR3Mono_GM.sf2";
    let soundfont = SoundFont::open(path)?;

    let engine = Engine::new()?;
    engine.play()?;
    
    // for (id, sample) in soundfont.samples.iter() {
    //     if !sample.name.contains("Piano") {
    //         continue;
    //     }
    //
    //     if !sample.name.contains("C4") {
    //         continue;
    //     }
    //
    //     let sender = engine.channel();
    //     for d in sample.data.iter() {
    //         sender.send(0.02 * *d)?;
    //     }
    //
    //     std::thread::sleep(Duration::from_secs_f64(5.0));
    // }
    
    // let mut clock = 0.0;
    //
    // loop {
    //     clock += 1. / 44100.;
    //     engine.channel().send(
    //         (clock * TAU * 440.).sin() * 0.02
    //     )?;
    // }

    std::thread::sleep(Duration::from_secs_f64(5.0));
    Ok(())
}
