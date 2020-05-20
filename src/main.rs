use anyhow::*;

#[derive(Debug,serde::Deserialize)] struct Room { rent: u32, address: String }

#[fehler::throws]
fn main() {
    let rooms : Vec<Room> = ron::de::from_reader(std::fs::File::open("../rooms")?)?;
    println!("{:?}", rooms);
}
