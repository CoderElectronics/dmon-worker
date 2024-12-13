use chrono::Utc;
use cron::Schedule;
use std::str::FromStr;

pub struct ScheduledTask {
    pub name: String,
    pub schedule: Schedule,
    pub task: Box<dyn Fn() -> Result<(), Box<dyn std::error::Error>> + Send + 'static>,
}

impl ScheduledTask {
    pub fn new<F>(
        name: &str,
        cron_expression: &str,
        task: F,
    ) -> Result<Self, Box<dyn std::error::Error>>
    where
        F: Fn() -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
    {
        Ok(ScheduledTask {
            name: name.to_string(),
            schedule: Schedule::from_str(cron_expression)?,
            task: Box::new(task),
        })
    }

    pub fn run(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("[{}] running at: {}", self.name, Utc::now());
        (self.task)()
    }
}
