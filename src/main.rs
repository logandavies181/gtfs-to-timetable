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

    let days_to_trips = get_days_to_trips(&gtfs, &trips_by_service_id)?;

    let t = days_to_trips.get("2021-01-20").expect("dangit!!");
    println!("Trips on 2021-01-20:");
    for i in t.iter() {
        println!("  {}", i.id);
    }

    let trip = gtfs.trips.get("CCL__1__10665__WCCL__CCL_Xmas_Period").expect("trip was fucked");
    println!("trip is {}", trip.id);

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

fn get_trips_by_service_id<'a>(gtfs: &'a Gtfs) ->  Result<HashMap<String, &'a Trip>> {
    let mut trips_by_service_id: HashMap<String, &Trip> = HashMap::new();
    for t in gtfs.trips.values() {
        trips_by_service_id.insert(t.service_id.clone(), t);
    }
    Ok(trips_by_service_id)
}

fn get_days_to_trips<'a>(gtfs: &'a Gtfs, trips_by_service_id: &'a HashMap<String, &'a Trip>) -> Result<HashMap<String, Vec<&'a Trip>>> {
    let mut days_to_trips: HashMap<String, Vec<&Trip>> = HashMap::new();
    for s in gtfs.calendar.values() {
        let service_id = s.id.as_str();
        let trip = trips_by_service_id.get(service_id).expect("trip missing");

        let days = get_days_for_service(&gtfs, service_id)?;
        for d in days.iter() {
            let date = d.date;
            // add date to map
            let date_str = format!("{}", date);
            let trips = days_to_trips.get_mut(date_str.as_str());
            match trips {
                Some(x) => {
                    x.push(trip);
                }
                None => {
                    days_to_trips.insert(date_str, vec![trip]);
                }
            }
        }
    }
    Ok(days_to_trips)
}
