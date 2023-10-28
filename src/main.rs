use serde::Deserialize;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::{thread, time};

const CONVERSION_FACTOR: f32 = 18.016;
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
    let current = get_ns_data().unwrap().get_range();
    let request_url = "https://nightscout.ransomti.me/api/v1/entries.json?count=100";
    let res: Vec<NsResults> = reqwest::blocking::get(request_url)
                               .unwrap().json().unwrap();
    let mut last_ts = res[0].to_timestamp();
    for e in res.iter() {
        if e.get_range() != current {
            return last_ts;
        }
        last_ts = e.to_timestamp();
    }
    get_ns_data().unwrap().to_timestamp()

}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("523786960947380245")?;
    println!("connecting to ns");

    let mut range = get_ns_data().unwrap().get_range();
    let mut last_change = get_last_change();

    println!("connecting to discord");
    client.connect()?;
    loop {
        let data = get_ns_data()?;
        if data.get_range() != range {
            last_change = data.to_timestamp();
            range = data.get_range();
        }
        client.set_activity(activity::Activity::new()
            .state(data.to_delta_str().as_str())
            .details(data.to_status_str().as_str())
            .timestamps(last_change.clone())
        )?;
        thread::sleep(time::Duration::from_secs(5));
    }
}

fn get_ns_data() -> Result<NsResults,reqwest::Error>{
    let request_url = "https://nightscout.ransomti.me/api/v1/entries.json?count=1";
    let res: Vec<NsResults> = reqwest::blocking::get(request_url)
                               .unwrap().json().unwrap();
    Ok(res[0].clone())
}
