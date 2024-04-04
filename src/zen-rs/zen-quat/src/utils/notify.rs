use cached::proc_macro::cached;
use notify_rust::Notification;
use time::{format_description, OffsetDateTime};
use zen_core::objects::trade::{Event, Factor, Signal};

pub struct Notify {}

impl Notify {
    pub fn notify_signal(dt: OffsetDateTime, signal: Signal) {
        let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]").unwrap();
        notify(
            signal.key(),
            Some(dt.format(&format).unwrap()),
            signal.value(),
            dt,
            true,
        );
    }

    pub fn notify_event(dt: OffsetDateTime, event: &Event, factor: &Factor) {
        let format = format_description::parse("[year]-[month]-[day] [hour]:[minute]").unwrap();
        notify(
            event.name.clone(),
            Some(dt.format(&format).unwrap()),
            factor
                .signals_all
                .iter()
                .map(|x| format!("{:?}", x))
                .collect::<Vec<_>>()
                .join("\n"),
            dt,
            false,
        );
    }
}

#[cached(size = 1000)]
fn notify(
    title: String,
    subtitle: Option<String>,
    body: String,
    dt: OffsetDateTime,
    realtime: bool,
) {
    if !realtime || dt.unix_timestamp() > OffsetDateTime::now_utc().unix_timestamp() - 60 * 10 {
        Notification::new()
            .summary(title.as_str())
            .subtitle(subtitle.unwrap_or("".to_string()).as_str())
            .body(body.as_str())
            .show()
            .unwrap();
    }
}
