use serde::Deserialize;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::{thread, time};
use chrono::prelude::*;

const CONVERSION_FACTOR: f32 = 18.016;
const ENDPOINT: &str = "https://nightscout.ransomti.me/api/v1/entries/sgv.json";
//const TOKEN: Option<&str> = None; //No token required
const TOKEN:Option<&str> = Some("reporter-eedc563cda3d55b7"); // a token

fn get_token_str() -> String{
    match TOKEN {
        Some(token) => format!("&token={token}"),
        None => String::new()
    }
}

#[derive(PartialEq, Eq, Copy, Clone)]
enum BgRange {
    High,
    Normal,
    Low,
}

#[derive(Deserialize, Debug, Clone)]
struct NsResults
{
    sgv: f32,
    delta: f32,
    direction: String,
    date: i64,
    #[serde(rename = "dateString")] 
    date_string: String,
}

impl NsResults {
    fn sgv_as_mmol(&self) -> f32 {
        self.sgv / CONVERSION_FACTOR
    }

    fn delta_as_mmol(&self) -> f32 {
        self.delta  / CONVERSION_FACTOR
    }

    fn direction_to_char(&self) -> char {
        match self.direction.to_ascii_uppercase().as_str() {
            "NONE" => '⇼',
            "TRIPLEUP" => '⤊',
            "DOUBLEUP"=> '⇈',
            "SINGLEUP"=> '↑',
            "FORTYFIVEUP"=> '↗',
            "FLAT" => '→',
            "FORTYFIVEDOWN"=> '↘',
            "SINGLEDOWN"=> '↓',
            "DOUBLEDOWN"=> '⇊',
            "TRIPLE DOWN"=> '⤋',
            "NOT COMPUTABLE"=> '-',
            "RATE OUT OF RANGE"=> '⇕',
            _ => unreachable!(),
        }
    }

    fn get_range(&self) -> BgRange {
        let mmol = self.sgv_as_mmol();
        if mmol > 10.0 {
            BgRange::High
        } else if mmol > 3.8 {
            BgRange::Normal
        } else {
            BgRange::Low
        }
    }

    fn to_status_str(&self) -> String {
        let range_state = match self.get_range() {
            BgRange::High => " (high)",
            BgRange::Low => " (low)",
            BgRange::Normal => "",
        };

        format!("{:.1} mmol/L{}",
        &self.sgv_as_mmol(),
        range_state,
        )
    }

    fn to_delta_str(&self) -> String {
        let add = if self.delta > 0.0 {
            '+'
        } else {
            '\0'
        };
        format!("{}{:.1} {}",
        add,
        &self.delta_as_mmol(),
        &self.direction_to_char())
    }

    fn to_timestamp(&self) -> activity::Timestamps {
        activity::Timestamps::new().start(self.date)
    }
}

fn get_last_change() -> activity::Timestamps {
    println!("fetching last change");
    let res: Vec<NsResults> = reqwest::blocking::get(format!("{ENDPOINT}?count=250{}",get_token_str()))
    .unwrap().json().unwrap();
    let mut last_ts = res[0].to_timestamp();
    let current = res[0].get_range();
    for e in res.iter() {
        if e.get_range() != current {
            break; //will return prev ts - the one before the change
        }
        last_ts = e.to_timestamp();
    }
    last_ts // will be 250th change if none found in this range

}

fn main() {


    loop {
        main_loop().unwrap_or_else(|e| {
            println!("Error: {}", e);
        });
        thread::sleep(time::Duration::from_secs(60));
    }

}

fn main_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("523786960947380245")?;
    println!("connecting to ns");

    let mut prev = get_ns_data()?;
    let mut last_change = get_last_change();

    println!("connecting to discord");
    client.connect()?;
    loop {
        let data = get_ns_data()?;
        if data.get_range() != prev.get_range() {
            last_change = data.to_timestamp();
        }

        let time_since = Utc::now().signed_duration_since(
            DateTime::parse_from_rfc3339(&data.date_string).unwrap()
        );

        if time_since.num_minutes() > 15 {
            client.set_activity(activity::Activity::new()
            .state("No value for 15 mins")
            .details(""))?;
        } else {
            client.set_activity(activity::Activity::new()
            .state(data.to_delta_str().as_str())
            .details(data.to_status_str().as_str())
            .timestamps(last_change.clone())
        )?;
        }
        if time_since.num_minutes() > 5 {
            thread::sleep(time::Duration::from_secs(5));
        } else {
            let to_sleep = 60 * 5 - time_since.num_seconds();

            thread::sleep(time::Duration::from_secs(to_sleep.try_into().unwrap_or(5)));
        }



        prev = data;
    }
}

fn get_ns_data() -> Result<NsResults,reqwest::Error>{
    let res: Vec<NsResults> = reqwest::blocking::get(format!("{ENDPOINT}?count=1{}",get_token_str()))
    .unwrap().json().unwrap();
    Ok(res[0].clone())
}
