use serde::{Deserialize,Serialize};
#[derive(Debug,Deserialize,Serialize)]
pub struct Coordinate(pub(crate) geo_types::Coordinate<f64>);
impl std::fmt::Display for Coordinate { fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> { write!(f, "{},{}", self.0.x, self.0.y) } }

use serde_json::Value;

#[derive(Debug,Deserialize)]
enum Code {
    Ok, // Request could be processed as expected.
    InvalidUrl, // URL string is invalid.
    InvalidService, // Service name is invalid.
    InvalidVersion, // Version is not found.
    InvalidOptions, // Options are invalid.
    InvalidQuery, // The query string is synctactically malformed.
    InvalidValue, // The successfully parsed query parameters are invalid.
    NoSegment, // One of the supplied input coordinates could not snap to street segment.
    TooBig, // The request size violates one of the service specific request size restrictions.
}

#[derive(Debug,Deserialize)]
#[allow(non_camel_case_types)]
enum Type {
    turn, // a basic turn into direction of the modifier
    new_name, // no turn is taken/possible, but the road name changes. The road can take a turn itself, following modifier .
    depart, // indicates the departure of the leg
    arrive, // indicates the destination of the leg
    merge, // merge onto a street (e.g. getting on the highway from a ramp, the modifier specifies the direction of the merge )
    on_ramp, // take a ramp to enter a highway (direction given my modifier )
    off_ramp, // take a ramp to exit a highway (direction given my modifier )
    fork, // take the left/right side at a fork depending on modifier
    end_of_road, // road ends in a T intersection turn in direction of modifier
    use_lane, // going straight on a specific lane
    r#continue, // Turn in direction of modifier to stay on the same road
    roundabout, // traverse roundabout, has additional field exit with NR if the roundabout is left. the modifier specifies the direction of entering the roundabout
    rotary, // Traffic circle. Large roundabout. Does not necessarily follow roundabout rules for right of way.
    roundabout_turn, // Describes a turn at a small roundabout that should be treated as normal turn. The modifier indicates the turn direciton.
    notification, // not an actual turn but a change in the driving conditions. For example the travel mode. If the road takes a turn itself, the modifier describes the direction
}

#[derive(Debug,Deserialize)]
#[allow(non_camel_case_types)]
enum Modifier { uturn, sharp_right, right, slight_right, straight, slight_left, left, sharp_left 	}

#[derive(Debug,Deserialize)]
struct StepManeuver {
    location: Coordinate,
    bearing_before: f32, // The clockwise angle from true north to the direction of travel immediately before the maneuver.
    bearing_after: f32, // The clockwise angle from true north to the direction of travel immediately after the maneuver.
    r#type: Type,
    modifier: Option<Modifier>, // indicates direction change
    exit: Option<u32>,
}

#[derive(Debug,Deserialize)]
struct Lane {
    indications: Vec<Modifier>,
    valid: bool,
}

#[derive(Debug,Deserialize)]
struct Intersection {
    location: Coordinate,
    bearings: Vec<u16>, // available roads
    entry: Vec<bool>, // //bearings. valid route or violate a restriction.
    r#in: u8, // index into bearings/entry array
    out: u8, // index into the bearings/entry array
    lanes: Vec<Lane>, // Available turn lanes at the intersection
}

#[derive(Debug,Deserialize)]
struct Waypoint {
    name: String, // Name of the street the coordinate snapped to
    location: Coordinate, // Snapped coordinate
    distance: f32, // Distance of the snapped point from the original
    hint: String, // internal UID of the segment (ephemeral, not constant over data updates) to optimize subsequent queries and connect multiple services (nearest -> route).
}

/// Manoeuvre such as a turn or merge, followed by a distance of travel along a single way to the subsequent step.
#[derive(Debug,Deserialize)]
struct RouteStep {
    distance: f32, // Distance of travel from the maneuver to the subsequent step, in float meters.
    duration: f32, // Estimated travel time, in float number of seconds.
    geometry: Value,
    name: String, // Name of the way along which travel proceeds.
    r#ref: Option<u32>, // Reference number or code for the way. Optionally included, if ref data is available for the given way.
    pronunciation: Option<String>, // Pronunciation hint of the way name. Will be undefined if there is no pronunciation hit.
    destinations: Vec<String>, // Destinations of the way. Will be undefined if there are no destinations.
    mode: String, // Mode of transportation.
    maneuver: StepManeuver,
    intersections: Vec<Intersection>
}

// Annotation of the whole route leg with fine-grained information about each segment or node id.
#[derive(Debug,Deserialize)]
struct Annotation {
    distance: f32, // Distance, in metres, between each pair of coordinates
    duration: f32, // Duration between each pair of coordinates, in seconds
    datasources: Vec<u32>, // Index of the datasource for the speed between each pair of coordinates (osrm-contract segment-speed-file)
    nodes: Vec<u32>, // OSM node ID for each coordinate along the route, excluding the first/last user-supplied coordinates
}

// Represents a route between two waypoints.
#[derive(Debug,Deserialize)]
struct RouteLeg {
    distance: f32, // Distance traveled by this route leg, in float meters.
    duration: f32, // Estimated travel time, in float number of seconds.
    summary: String,
    steps: Vec<RouteStep>,
    annotation: Option<Annotation>,
}

// Route through (potentially multiple) waypoints.
#[derive(Debug,Deserialize)]
pub struct Route {
    pub distance: f32, // The distance traveled by the route, in float meters.
    duration: f32, // The estimated travel time, in float number of seconds.
    geometry: Value,
    legs: Vec<RouteLeg>,
}

#[derive(Debug,Deserialize)]
pub struct Response {
    code: Code,
    message: Option<String>,
    waypoints: Vec<Waypoint>,
    pub routes: Vec<Route>,
}
