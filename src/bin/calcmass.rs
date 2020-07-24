use nastran::bdf::{deck::Deck, Result};
use std::io;
use std::time::Instant;

pub fn main() -> Result<()> {
    let mut args = std::env::args();
    let _ = args
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing argument"))?;
    let filename = args
        .next()
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "missing argument"))?;
    println!("{}", filename);
    let t = Instant::now();
    //let deck = Deck::from_bytes(bytes.into_iter().map(Ok))?;
    let deck = Deck::from_filename(filename)?;
    println!("Parse took {} ms", t.elapsed().as_millis());
    let t = Instant::now();
    let global = deck.global_locations();
    println!("Coordinates took {} ms", t.elapsed().as_millis());
    let t = Instant::now();
    let (mass, cg) = deck.mass_cg(&global);
    println!("Mass CG took {} ms", t.elapsed().as_millis());
    println!("mass = {:8.2}", mass);
    println!("x_cg = {:8.2}", cg.x());
    println!("y_cg = {:8.2}", cg.y());
    println!("z_cg = {:8.2}", cg.z());
    Ok(())
}
