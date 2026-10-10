use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timing {
    pub connect: Duration,
    pub connect_poll: Duration,
    pub command: Duration,
    pub drain: Duration,
    pub navigate: Duration,
    pub settle: Duration,
    pub poll: Duration,
    pub wait: Duration,
    pub actionable: Duration,
    pub network_idle: Duration,
    pub close: Duration,
}

impl Default for Timing {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(45),
            connect_poll: Duration::from_millis(250),
            command: Duration::from_secs(30),
            drain: Duration::from_millis(25),
            navigate: Duration::from_secs(15),
            settle: Duration::from_millis(2500),
            poll: Duration::from_millis(120),
            wait: Duration::from_secs(30),
            actionable: Duration::from_millis(1000),
            network_idle: Duration::from_millis(500),
            close: Duration::from_millis(1500),
        }
    }
}
