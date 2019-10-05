use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

pub const SEED_BITS: i64 = 5;
pub const SOIL_BITS: i64 = 5;
pub const HARVESTED_COUNT_BITS: i64 = 12;

const MAX_SEED: i64 = -1 ^ (-1 << SEED_BITS);
const MAX_SOIL: i64 = -1 ^ (-1 << SOIL_BITS);

const SOIL_SHIFT: i64 = HARVESTED_COUNT_BITS;
const SEED_SHIFT: i64 = HARVESTED_COUNT_BITS + SEED_BITS;
const UNIX_TIME_SHIFT: i64 = HARVESTED_COUNT_BITS + SOIL_BITS + SEED_BITS;

const HARVESTED_COUNT_MASK: i64 = -1 ^ (-1 << HARVESTED_COUNT_BITS);

pub struct Snowberry {
    seed: i64,
    soil: i64,

    harvested_count: i64,
    last_harvested_at: i64,
}

impl Snowberry {
    pub fn new(seed: i64, soil: i64) -> Self {
        assert!(0 <= seed && seed <= MAX_SEED);
        assert!(0 <= soil && soil <= MAX_SOIL);

        Snowberry {
            seed: seed << SEED_SHIFT,
            soil: soil << SOIL_SHIFT,
            harvested_count: 0,
            last_harvested_at: 0,
        }
    }

    pub fn harvest(&mut self) -> i64 {
        self.harvest_from_time(&self.wait_for_next_harvest())
            .expect("harvest failed to wait next")
    }

    pub fn harvest_from_time(&mut self, time: &SystemTime) -> Result<i64, &str> {
        let unix_time = self.to_unix_time(&time);
        assert!(self.last_harvested_at <= unix_time);

        if unix_time == self.last_harvested_at {
            self.harvested_count = (self.harvested_count + 1) & HARVESTED_COUNT_MASK;

            if self.harvested_count == 0 {
                return Err("snowberry has harvested all");
            }
        } else {
            self.harvested_count = 0;
        }

        self.last_harvested_at = unix_time;
        Ok((unix_time << UNIX_TIME_SHIFT) | self.seed | self.soil | self.harvested_count)
    }

    fn to_unix_time(&self, time: &SystemTime) -> i64 {
        let dur =
        time.duration_since(UNIX_EPOCH)
            .expect("failed to generate unix time from SystemTime");

        dur.as_millis() as i64
    }

    fn wait_for_next_harvest(&self) -> SystemTime {
        loop {
            let now = SystemTime::now();
            let unix_time = self.to_unix_time(&now);
            if self.last_harvested_at < unix_time {
                return now;
            }

            thread::sleep(Duration::from_millis(1));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Snowberry;
    use crate::{HARVESTED_COUNT_BITS, MAX_SEED, MAX_SOIL};
    use std::collections::HashSet;
    use std::time::SystemTime;

    #[test]
    fn it_works() {
        let mut s = Snowberry::new(30, 13);
        let id = s.harvest();
        assert!(0 < id, format!("{}", id));
    }

    #[test]
    fn harvest_generates_uniq_berry() {
        let mut ids = HashSet::new();
        for seed in 0..MAX_SEED {
            for soil in 0..MAX_SOIL {
                let mut s = Snowberry::new(seed, soil);

                for t in 0..5 {
                    let id = s.harvest();
                    assert!(0 < id, format!("{}", id));
                    assert!(
                        !ids.contains(&id),
                        format!("failed to gen uniq at count: {}, id: {}", t, id)
                    );
                    ids.insert(id);
                }
            }
        }
    }

    #[test]
    fn harvest_from_time_generates_uniq_berry() {
        let mut ids = HashSet::new();
        let now = SystemTime::now();
        let mut s = Snowberry::new(0, 0);

        for t in 0..(1 << HARVESTED_COUNT_BITS) {
            let id = s.harvest_from_time(&now).unwrap();

            assert!(0 < id, format!("{}", id));
            assert!(
                !ids.contains(&id),
                format!("failed to gen uniq at count: {}, id: {}", t, id)
            );
            ids.insert(id);
        }
    }
}
