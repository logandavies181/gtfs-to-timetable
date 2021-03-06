//use std::fs::write;
use std::collections::{HashMap, HashSet};
use std::time::Instant;

use gtfs_structures::*;
use serde::{Deserialize, Serialize};
use topological_sort::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

struct Dat {
    g: Gtfs,
    sid_to_dates: HashMap<String, Vec<String>>,
    date_to_sids: HashMap<String, Vec<String>>,
    sid_to_trips: HashMap<String, Vec<String>>,
    route_orders: HashMap<String, Vec<String>>,
}

impl Dat {
    fn from_gtfs(g: Gtfs) -> Dat {
        let mut dat = Dat {
            g: g,
            sid_to_dates: HashMap::new(),
            date_to_sids: HashMap::new(),
            sid_to_trips: HashMap::new(),
            route_orders: HashMap::new(),
        };
        dat.order_sid_to_dates();
        dat.order_sid_to_trips();
        dat.get_route_orders();
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
    fn get_route_orders(&mut self) {
        let mut route_id_to_trips: HashMap<String, Vec<&Trip>> = HashMap::new();
        let mut route_id_to_inbound_only_trips: HashMap<String, Vec<&Trip>> = HashMap::new();
        let mut inbound_only_routes = HashSet::new();
        for trip in self.g.trips.values() {
            match trip.direction_id {
                Some(x) => {
                    match x {
                        DirectionType::Outbound => push_value_to_hashmap_vec(&mut route_id_to_trips, trip.route_id.clone(), trip),
                        DirectionType::Inbound => {},
                    }
                },
                _ => panic!("could not sort stops due to not knowing direction of travel"),
            }
        }

        // Get trips for routes that are inbound only
        for route_id in self.g.routes.keys() {
            if !route_id_to_trips.contains_key(route_id.as_str()) {
                inbound_only_routes.insert(route_id.clone());
            }
        }
        for trip in self.g.trips.values() {
            if inbound_only_routes.contains(&trip.route_id) {
                match trip.direction_id {
                    Some(x) => {
                        match x {
                            DirectionType::Outbound => panic!(format!("thought that route {} was inbound only but found outbound route", &trip.route_id)),
                            DirectionType::Inbound => {
                                push_value_to_hashmap_vec(&mut route_id_to_inbound_only_trips, trip.route_id.clone(), trip);
                            },
                        }
                    },
                    _ => panic!("Somehow got a direction before but not now"),
                }
            }
        }

        for (route_id, trips) in route_id_to_trips.iter() {
            let order = get_outbound_trip_order(trips.to_vec(), route_id.clone());
            self.route_orders.insert(route_id.clone(), order);
        }

        for (route_id, trips) in route_id_to_inbound_only_trips.iter() {
            println!("trying inbound for {}", route_id);
            let mut order = get_outbound_trip_order(trips.to_vec(), route_id.clone());
            order.reverse();
            self.route_orders.insert(route_id.clone(), order);
        }
    }
}

fn get_outbound_trip_order(trips: Vec<&Trip>, route_id: String) -> Vec<String> {
        //let mut dependencies = HashSet::new();
        let mut topologicalsort: TopologicalSort<String> = TopologicalSort::new();
        for trip in trips.iter() {
            let mut prev: &StopTime = &trip.stop_times[0];
            for count in 0..trip.stop_times.len()-1 {
                let stime = &trip.stop_times[count];
                if count == 0 {
                    prev = stime;
                    continue;
                }
                topologicalsort.add_dependency(stime.stop.id.clone(), prev.stop.id.clone());
                prev = stime;
            }
        }
        let mut order: Vec<String> = Vec::new();
        while topologicalsort.len() != 0 {
            let next_opt = topologicalsort.pop();
            match next_opt {
                Some(x) => order.push(x),
                None => {
                    println!("could not sort route {}", route_id);
                    return order
                },
            }
        }

        order
}

fn push_value_to_hashmap_vec<T>(hash: &mut HashMap<String, Vec<T>>, key: String, value: T) {
    let vec_opt = hash.get_mut(key.as_str());
    match vec_opt {
        Some(v) => v.push(value),
        None => {
            hash.insert(key, vec![value]);
        }
    }
}

// Parse u32 number of seconds into HH:MM
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
    inbound_order: Vec<String>,
    outbound_order: Vec<String>,
}

impl RouteInfo {
    fn new() -> RouteInfo {
        RouteInfo {
            route_id: String::from(""),
            inbound: Vec::new(),
            outbound: Vec::new(),
            inbound_order: Vec::new(),
            outbound_order: Vec::new(),
        }
    }
}

#[derive(Serialize, Deserialize)]
struct TripInfo {
    times: HashMap<String, String>,
    first_time: u32,
}

impl TripInfo {
    fn new() -> TripInfo {
        TripInfo {
            times: HashMap::new(),
            first_time: std::u32::MAX,
        }
    }
}

// Gets all of the stop times and direction from a trip and adds it to the corresponding
// key in the map
fn add_trip_to_route_info(riri: &mut HashMap<String, RouteInfo>, trip: &Trip) {
    let mut trip_info = TripInfo::new();

    let mut first_time: u32 = std::u32::MAX;
    // FIXME: consider departure_time also
    for stime in trip.stop_times.iter() {
        let arrival_time = stime.arrival_time.expect("No arrival time");
        if arrival_time < first_time {
            first_time = arrival_time;
        }
        trip_info.times.insert(stime.stop.id.clone(), parse_time(arrival_time));
    }
    trip_info.first_time = first_time;

    let route_id = trip.route_id.clone();
    let route_info = riri.entry(route_id.clone());
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
        std::collections::hash_map::Entry::Occupied(mut ri) => {
            match trip.direction_id {
                Some(DirectionType::Inbound) => {
                    ri.get_mut().inbound.push(trip_info);
                },
                Some(DirectionType::Outbound) => {
                    ri.get_mut().outbound.push(trip_info);
                },
                None => panic!("ahhhhh")
            };
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

    // Loop over each date and then get all services that run on that day to get all the trips
    // that run on that day
    for (date, sids) in dat.date_to_sids {
        let mut route_id_to_route_info: HashMap<String, RouteInfo> = HashMap::new();
        for sid in sids {
            for trip_id in dat.sid_to_trips.get(sid.as_str()).expect("failed2").iter() {
                let trip = dat.g.trips.get(trip_id.as_str()).expect("failed3");
                add_trip_to_route_info(&mut route_id_to_route_info, trip);
            }
        }

        // Print all of the trips for a route on a given day to a file
        for (key, value) in route_id_to_route_info.iter_mut() {
            let stop_order_opt = dat.route_orders.get(key.clone().as_str());
            match stop_order_opt {
                Some(x) => value.stop_order = x.to_vec(),
                None => {
                    //println!("{:?}", dat.route_orders);
                },
            }
            let data = serde_json::to_string(&value)?;
            std::fs::create_dir_all(format!("output/{}", date))?;
            std::fs::write(format!("output/{}/{}.json", date, key), data)?;
        }
    }

    Ok(())
}

/* TODO:
 * - add handling for 1s in calendar.txt
 * - add handling for dates in calendar.txt
 */
