use std::rc::Rc;
use std::{env, fs};

use crate::objects::trade::Matcher;
use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use tracing::debug;

#[derive(Debug, Deserialize, Copy, Clone)]
pub(crate) enum BiType {
    Modern,
    Legacy,
    FourK,
}

#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    pub debug: bool,
    pub bi_type: BiType,
    pub bi_change_threshold: f32,
    pub max_bi_num: usize,
    pub event_matcher_file: String,
    #[serde(skip_deserializing)]
    #[serde(skip_serializing)]
    pub matcher: Option<Rc<Matcher>>,
}

impl Settings {
    pub fn new() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "dev".into());

        let s = Config::builder()
            // Start off by merging in the "default" configuration file
            .add_source(File::with_name("./config/default"))
            // Add in the current environment file
            // Default to 'development' env
            // Note that this file is _optional_
            .add_source(File::with_name(&format!("./config/{}", run_mode)).required(false))
            // Add in a local configuration file
            // This file shouldn't be checked in to git
            .add_source(File::with_name("./config/local").required(false))
            // Add in settings from the environment (with a prefix of APP)
            // Eg.. `APP_DEBUG=1 ./target/app` would set the `debug` key
            .add_source(Environment::with_prefix("zen"))
            // You may also programmatically change settings
            .build()?;

        // You can deserialize (and thus freeze) the entire configuration as
        let mut s: Settings = s.try_deserialize()?;

        let content =
            fs::read_to_string(s.event_matcher_file.as_str()).expect("TODO: panic message");
        let events = Matcher::from(content.as_str()).expect("panic");
        s.matcher = Some(Rc::new(events));
        debug!("settings:\n {:?}", s);
        Ok(s)
    }
}
