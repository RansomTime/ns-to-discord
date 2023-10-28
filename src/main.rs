use serde::Deserialize;
use discord_rich_presence::{activity, DiscordIpc, DiscordIpcClient};
use std::{thread, time};

const CONVERSION_FACTOR: f32 = 18.016;

#[derive(Deserialize, Debug, Clone)]
struct NsResults
{
    sgv: i32,
    delta: i32,
    direction: String,
    date: i64,
}

impl NsResults {
    fn sgv_as_mmol(&self) -> f32 {
        self.sgv as f32 / CONVERSION_FACTOR
    }

    fn delta_as_mmol(&self) -> f32 {
        self.delta as f32 / CONVERSION_FACTOR
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

    fn to_status_str(&self) -> String {
        format!("{:.1} mmol/L",
            &self.sgv_as_mmol())
    }

    fn to_delta_str(&self) -> String {
        let add = if self.delta > 0 {
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




fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut client = DiscordIpcClient::new("523786960947380245")?;
    client.connect()?;
    loop {
        let data = get_ns_data()?;
        client.set_activity(activity::Activity::new()
            .state(data.to_delta_str().as_str())
            .details(data.to_status_str().as_str())
            .timestamps(data.to_timestamp())
        )?;
        thread::sleep(time::Duration::from_secs(15));
    }
}

fn get_ns_data() -> Result<NsResults,reqwest::Error>{
    let request_url = "https://nightscout.ransomti.me/api/v1/entries/sgv?count=1";
    let res: Vec<NsResults> = reqwest::blocking::get(request_url)
                               .unwrap().json().unwrap();
    Ok(res[0].clone())
}
