use anyhow::Result;
use headless_chrome::{protocol::cdp::Page::CaptureScreenshotFormatOption, Browser, LaunchOptions};
use std::fs;
use tracing::{debug, error};

pub fn screenshot(path: String, outdir: String) -> Result<()> {
    let ct = fs::read_to_string(path)?;
    let options = LaunchOptions::default_builder()
        .window_size(Some((3840, 2160)))
        .build()
        .expect("Couldn't find appropriate Chrome binary.");
    let browser = Browser::new(options)?;

    for (idx, line) in ct.lines().enumerate() {
        let symbol = line.split_whitespace().collect::<Vec<_>>()[0];
        let exchange = line.split_whitespace().rev().collect::<Vec<_>>()[0];
        if exchange != "美股" || symbol.starts_with(".") {
            continue;
        }

        debug!("crawling {}-{}", exchange, symbol);
        let res = || -> Result<()> {
            let tab = browser.new_tab()?;
            let jpeg_data = tab
                .navigate_to(
                    format!("http://localhost:3000/local?symbolState=\"{}\"", symbol).as_str(),
                )?
                .wait_until_navigated()?
                .capture_screenshot(CaptureScreenshotFormatOption::Jpeg, Some(75), None, true)?;
            fs::write(
                format!(
                    "./{}/{:03}-{}.jpg",
                    outdir.trim_end_matches("/"),
                    idx,
                    symbol
                ),
                jpeg_data,
            )?;
            tab.close(true)?;
            Ok(())
        }();
        if res.is_err() {
            error!("{} error {}", line, res.unwrap_err())
        }
    }

    Ok(())
}
