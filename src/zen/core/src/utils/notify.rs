use cached::proc_macro::cached;
use chrono::{Duration, Local};
use notify_rust::Notification;
use crate::analyze::Symbol;
use crate::element::chan::DT;
use crate::element::event::{Event, Factor, Signal};

pub struct Notify {}

impl Notify {
    pub fn notify_signal(symbol: &Symbol, dt: DT, signal: Signal) {
        notify(
            format!("{} - {}", symbol, signal.key()),
            Some(format!("{}", dt.format("%Y-%m-%d %H:%M"))),
            signal.value(),
            dt,
            true,
        );
    }

    pub fn notify_event(symbol: &Symbol, dt: DT, event: &Event, factor: &Factor) {
        notify(
            format!("{} - {}", symbol, event.name.clone()),
            Some(format!("{}", dt.format("%Y-%m-%d %H:%M"))),
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
    dt: DT,
    realtime: bool,
) {
    if !realtime || dt > Local::now() - Duration::hours(2) {
        Notification::new()
            .summary(title.as_str())
            .subtitle(subtitle.unwrap_or("".to_string()).as_str())
            .body(body.as_str())
            .sound_name("Submarine")
            .show()
            .unwrap();
    }
}
