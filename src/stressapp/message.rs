use chrono::{DateTime, NaiveTime, Utc};

#[derive(Debug, Clone)]
pub struct BasicMessage {
    pub server_id: u8,
    pub stress_tester: u8,
    pub percentage: f32,
    pub timestamp: DateTime<Utc>,
}

pub fn parse_message(input: &str) -> Option<BasicMessage> {
    // Parse format: [0-2]-[0-4]-[float<100]-hh:mm:ss
    let parts: Vec<&str> = input.split('-').collect();
    if parts.len() != 4 {
        return None;
    }

    let server_id = parts[0].parse::<u8>().ok()?;
    let stress_tester = parts[1].parse::<u8>().ok()?;
    let percentage = parts[2].parse::<f32>().ok()?;
    let timestamp = NaiveTime::parse_from_str(parts[3], "%H:%M:%S").ok()?;

    // Validate ranges
    if server_id > 2 || stress_tester > 4 || percentage < 0.0 || percentage > 100.0 {
        return None;
    }

    Some(BasicMessage {
        server_id,
        stress_tester,
        percentage,
        timestamp: Utc::now().date_naive().and_time(timestamp).and_utc(),
    })
}

#[derive(Debug, Clone)]
pub enum AppMessage {
    NewDataPoint(BasicMessage),
    Tick,
}
