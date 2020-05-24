#![feature(never_type)]
mod wg;
use {derive_more::Deref, fehler::throws, anyhow::{Error, bail}, chrono::NaiveTime, serde::{Deserialize, Deserializer}};

#[derive(Debug)] struct Time(NaiveTime); //chrono::DateTime<Local>);
impl<'de> Deserialize<'de> for Time {
    #[throws(D::Error)]
    fn deserialize<D:Deserializer<'de>>(deserializer: D) -> Self {
        Time(NaiveTime::parse_from_str(&String::deserialize(deserializer)?, "%H:%M").map_err(serde::de::Error::custom)?)
    }
}

#[derive(Debug,Deserialize,Deref)] struct Room { #[deref] address: String, rent: u32 }
#[derive(Debug,Deserialize)] enum Trip { From(Time), To(Time) }
#[derive(Debug,Deserialize,Deref)] struct Goal { #[deref] address: String, trips: Vec<Trip> }

#[throws]
fn main() -> ! {
    bail!("{:?}", wg::search());

    /*let rooms : Vec<Room> = ron::de::from_reader(std::fs::File::open("../rooms.ron")?)?;
    let goals: Vec<Goal> = ron::de::from_reader(std::fs::File::open("../goals.ron")?)?;
    use {map::{location, route], itertools::Itertools};
    println!("{:?}", rooms.iter().map(|room| goals.iter().map(move |goal| map::route(&[location(room), location(goal)]).unwrap().routes[0].distance).format(" ")).format("\n"));*/
}
