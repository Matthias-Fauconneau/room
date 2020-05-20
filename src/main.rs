use {fehler::throws, anyhow::Error, chrono::NaiveTime, serde::{Deserialize,Deserializer}};

#[derive(Debug)] struct Time(NaiveTime); //chrono::DateTime<Local>);
impl<'de> Deserialize<'de> for Time {
    #[throws(D::Error)]
    fn deserialize<D:Deserializer<'de>>(deserializer: D) -> Self {
        Time(NaiveTime::parse_from_str(&String::deserialize(deserializer)?, "%H:%M").map_err(serde::de::Error::custom)?)
    }
}

#[derive(Debug,Deserialize)] struct Room { address: String, rent: u32 }
#[derive(Debug,Deserialize)] enum Trip { From(Time), To(Time) }
#[derive(Debug,Deserialize)] struct Goal { address: String, trips: Vec<Trip> }

#[throws]
fn main() {
    let rooms : Vec<Room> = ron::de::from_reader(std::fs::File::open("../rooms.ron")?)?;
    let goals: Vec<Goal> = ron::de::from_reader(std::fs::File::open("../goals.ron")?)?;
    println!("{:?} {:?}", rooms, goals);
}
