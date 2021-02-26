//use std::fs::write;
use std::collections::HashMap;
use std::time::Instant;

use gtfs_structures::*;
use serde::{Deserialize, Serialize};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Dat {
    g: Gtfs,
    sid_to_dates: HashMap<String, Vec<String>>,
    date_to_sids: HashMap<String, Vec<String>>,
    sid_to_trips: HashMap<String, Vec<String>>,
}

impl Dat {
    fn from_gtfs(g: Gtfs) -> Dat {
        let mut dat = Dat {
            g: g,
            sid_to_dates: HashMap::new(),
            date_to_sids: HashMap::new(),
            sid_to_trips: HashMap::new(),
        };
        dat.order_sid_to_dates();
        dat.order_sid_to_trips();
        dat
    }
}

impl Dat {
    // Assumes all days are 0 in calendar.txt
    // TODO: remove assumptions
    fn order_sid_to_dates(&mut self) {
        for (id, src_vec) in self.g.calendar_dates.iter_mut() {
            let key = id.as_str();
            let mut vec_to_push: Vec<String> = Vec::new();
            for cal_date in src_vec.iter() {
                match cal_date.exception_type {
                    Exception::Added => {
                        let datestring = format!("{}", cal_date.date);
                        vec_to_push.push(datestring.clone());
                        // add sid to date_to_sids
                        push_value_to_hashmap_vec(&mut self.date_to_sids, datestring, String::from(key));
                    }
                    Exception::Deleted => {},
                }
            }
            let vec_from_hashmap = self.sid_to_dates.get_mut(key);
            match vec_from_hashmap {
                Some(v) => {
                    v.append(&mut vec_to_push);
                }
                None => {
                    self.sid_to_dates.insert(String::from(key), Vec::new());
                }
            }
        }
    }
    fn order_sid_to_trips(&mut self) {
        for trip in self.g.trips.values() {
            push_value_to_hashmap_vec(&mut self.sid_to_trips, trip.service_id.clone(), trip.id.clone())
        }
    }
}

fn push_value_to_hashmap_vec(hash: &mut HashMap<String, Vec<String>>, key: String, value: String) {
    let vec_opt = hash.get_mut(key.as_str());
    match vec_opt {
        Some(v) => v.push(value),
        None => {
            hash.insert(key, vec![value]);
        }
    }
}

fn parse_time(seconds: u32) -> String {
    let mut minutes = seconds / 60;
    let hours = minutes / 60;
    minutes = minutes - hours * 60;


    if minutes < 10 {
        String::from(format!("{}:0{}", hours, minutes))
    } else {
        String::from(format!("{}:{}", hours, minutes))
    }
}

#[derive(Serialize, Deserialize)]
struct RouteInfo {
    route_id: String,
    inbound: Vec<TripInfo>,
    outbound: Vec<TripInfo>,
}

impl RouteInfo {
    fn new() -> RouteInfo {
        RouteInfo {
            route_id: String::from(""),
            inbound: Vec::new(),
            outbound: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TripInfo {
    trip_id: String,
    times: HashMap<String, String>,
}

impl TripInfo {
    fn new() -> TripInfo {
        TripInfo {
            trip_id: String::from(""),
            times: HashMap::new(),
        }
    }
}

fn main() -> Result<()> {
    println!("Parsing gtfs data...");
    let start = Instant::now();
    let gtfs = Gtfs::from_path("./files/")?;
    println!("Parsed gtfs files in: {}s", start.elapsed().as_secs());

    let parse_dat_time = Instant::now();
    let dat = Dat::from_gtfs(gtfs);
    println!("Parsed gtfs data in: {}s", parse_dat_time.elapsed().as_secs());

    let services = dat.date_to_sids.get("2021-01-20").expect("failed");
    for s in services.iter() {
        println!("{}", s);
    }

    let mut route_id_to_route_info: HashMap<String, RouteInfo> = HashMap::new();
    for tripid in dat.sid_to_trips.get("Rail MTuWThF-XHol").expect("failed2").iter(){
        let trip = dat.g.trips.get(tripid.as_str()).expect("failed3");
        //trip_info.direction = format!("{:?}", trip.direction_id.expect(""));
        //trip_info.route_id = trip.route_id.clone();
        add_trip_to_route_info(&mut route_id_to_route_info, trip);

    }

    let j = serde_json::to_string(&route_id_to_route_info)?;

    println!("{}", j);

    Ok(())
}

fn add_trip_to_route_info(riri: &mut HashMap<String, RouteInfo>, trip: &Trip) {
        let mut trip_info = TripInfo::new();
        trip_info.trip_id = trip.id.clone();

        for stime in trip.stop_times.iter() {
            trip_info.times.insert(parse_time(stime.arrival_time.expect("failed4")), stime.stop.name.clone());
        }

        let trip_id = trip.id.clone();
        let route_id = trip.route_id.clone();
        let route_info = riri.entry(trip_id);
        match route_info {
            std::collections::hash_map::Entry::Vacant(_) => {
                let mut new_route_info = RouteInfo::new();
                new_route_info.route_id = route_id.clone();
                match trip.direction_id {
                    Some(DirectionType::Inbound) => {
                        new_route_info.inbound.push(trip_info);
                        riri.insert(route_id.clone(), new_route_info);
                    },
                    Some(DirectionType::Outbound) => {
                        new_route_info.outbound.push(trip_info);
                        riri.insert(route_id.clone(), new_route_info);
                    },
                    None => panic!("ahhhhh")
                };
            },
            std::collections::hash_map::Entry::Occupied(_) => {

            }
        }
}

/* TODO:
 * - add handling for 1s in calendar.txt
 * - add handling for dates in calendar.txt
 */
