use makeshift_instrument::soundfont::SoundFont;


fn main() -> anyhow::Result<()> {
    let path = "/home/creme_brulee/Downloads/198_Yamaha_SY1_piano.sf2";
    let sf = SoundFont::open_path(path)?;
    println!("{:?}", sf.sfbk);

    Ok(())
}
