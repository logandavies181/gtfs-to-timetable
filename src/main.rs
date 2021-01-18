//use std::fs::write;
use std::time::Instant;
use std::collections::HashMap;

//use chrono::NaiveDate;
use gtfs_structures::*;
use simple_error::*;

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    println!("Parsing gtfs data...");
    let start = Instant::now();
    let gtfs = Gtfs::from_path("./files/")?;
    println!("Parsed gtfs files in: {}s", start.elapsed().as_secs());

    let trips_by_service_id = get_trips_by_service_id(&gtfs)?;
    println!("length trips: {}, length trips_by_service_id: {}", gtfs.trips.len(), trips_by_service_id.len());

    let days_to_trips = get_days_to_trips(&gtfs, &trips_by_service_id)?;

    let _ = render_timetable(days_to_trips, "2021-01-20", "Rail MTuWThF-XHol");

    Ok(())
}

/* TODO:
 * - add handling for 1s in calendar.txt
 * - add handling for dates in calendar.txt
 */

fn get_days_for_service<'a>(gtfs: &'a Gtfs, service_id: &str) -> Result<&'a Vec<CalendarDate>> {
    let v = gtfs.calendar_dates.get(service_id);
    match v {
        Some(x) => Ok(x),
        None => bail!("woops"),
    }
}

fn get_trips_by_service_id<'a>(gtfs: &'a Gtfs) ->  Result<HashMap<String, Vec<&'a Trip>>> {
    // loop over this so we know all trips are consumed
    let mut trips: Vec<&Trip> = Vec::new();
    for t in gtfs.trips.values() {
        trips.push(t.clone());
    }

    // Populate each key of return value with empty Vec to fill up later
    let mut trips_by_service_id: HashMap<String, Vec<&Trip>> = HashMap::new();
    // TODO handle calendar_dates as needed
    for c in gtfs.calendar.values() {
        trips_by_service_id.insert(c.id.clone(), vec![]);
    }

    for c in gtfs.calendar.values() {
        for t in trips.iter() {
            if t.service_id == c.id {
                trips_by_service_id.get_mut(c.id.as_str())
                    .expect("failed to get tripvec for map")
                    .push(t);
            }
        }
    }

    Ok(trips_by_service_id)
}

fn get_days_to_trips<'a>(gtfs: &'a Gtfs, trips_by_service_id: &'a HashMap<String, Vec<&'a Trip>>) -> Result<HashMap<String, Vec<&'a Trip>>> {
    // Populate return value with empty vecs
    let mut days_to_trips: HashMap<String, Vec<&Trip>> = HashMap::new();

    for s in gtfs.calendar.values() {
        let service_id = s.id.as_str();
        let trips = trips_by_service_id.get(service_id).expect("trip missing");

        let days = get_days_for_service(&gtfs, service_id)?;
        for d in days.iter() {
            let date = d.date;
            // add date to map
            let date_str = format!("{}", date);
            let trips_on_day = days_to_trips.get_mut(date_str.as_str());
            match trips_on_day {
                Some(x) => {
                    x.append(&mut trips.clone());
                }
                None => {
                    days_to_trips.insert(date_str, trips.clone());
                }
            }
        }
    }

    Ok(days_to_trips)
}

fn render_timetable<'a>(days_to_trips: HashMap<String, Vec<&'a Trip>>, date: &str, service_id: &str) -> Result<String> {
    // TODO filter before this fn. Also filter by route_id

    let mut trips = days_to_trips.get(date).expect("messed up rendering trips").clone();
    trips = filter_trips_by_service_id(trips, service_id);
    println!("Service: {}", service_id);
    println!("Date: {}", date);
    for i in trips.iter() {
        println!("Times for {}:", i.id);
        for j in i.stop_times.iter() {
            match j.arrival_time {
                Some(t) => println!("{}", t),
                None => println!("No arrival time!!"),
            }
        }
    }

    Ok(String::from("foo"))
}

fn filter_trips_by_service_id<'a>(trips: Vec<&'a Trip>, service_id: &str) -> Vec<&'a Trip> {
    let mut xtrips = trips.clone();
    let mut i = 0;

    while i != xtrips.len() {
        // TODO remove hardcoded route filter
        if xtrips[i].service_id != service_id {
            xtrips.remove(i);
        } else {
            i += 1;
        }
    }

    xtrips
}
