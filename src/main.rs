#![feature(never_type)]

use {std::fmt::Debug, anyhow::Context, itertools::Itertools};
#[throws] fn process_results<T,E,I:Iterator<Item=Result<T,E>>,R,F:FnOnce(itertools::ProcessResults<&mut I, E>)->R>(mut iter: I, processor: F) -> R
where Result<R,E>:Context<R,E>, Result<T,E>:Debug { itertools::process_results::<_,F,T,E,R>(iter.by_ref(), processor).context(format!("{:?}", iter.format("\n")))? }

use {fehler::throws, chrono::NaiveTime};
#[derive(Debug)] struct Time(NaiveTime); //chrono::DateTime<Local>);
impl<'de> Deserialize<'de> for Time {
    #[throws(D::Error)]
    fn deserialize<D:Deserializer<'de>>(deserializer: D) -> Self {
        Time(NaiveTime::parse_from_str(&String::deserialize(deserializer)?, "%H:%M").map_err(serde::de::Error::custom)?)
    }
}

use {derive_more::Deref, serde::{Deserialize, Deserializer}};
#[derive(Debug,Deserialize,Deref)] struct Room { #[deref] address: String, rent: u32 }
#[derive(Debug,Deserialize)] enum Trip { From(Time), To(Time) }
#[derive(Debug,Deserialize,Deref)] struct Goal { #[deref] address: String, trips: Vec<Trip> }

mod wg;

use anyhow::Error;
#[throws]
fn main() {
    process_results(wg::rooms()?, |rooms| {
        let rooms : Vec<_> = rooms.sorted().take(16).collect();
        eprintln!("{}", rooms.iter().format("\n"));
        println!("{}", rooms.iter().map(|r| r.url()).format("\n"));
    } )?

    /*let rooms : Vec<Room> = ron::de::from_reader(std::fs::File::open("../rooms.ron")?)?;
    let goals: Vec<Goal> = ron::de::from_reader(std::fs::File::open("../goals.ron")?)?;
    use {map::{location, route], itertools::Itertools};
    println!("{:?}", rooms.iter().map(|room| goals.iter().map(move |goal| map::route(&[location(room), location(goal)]).unwrap().routes[0].distance).format(" ")).format("\n"));*/
}
