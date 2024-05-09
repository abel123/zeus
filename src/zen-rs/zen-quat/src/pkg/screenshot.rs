use anyhow::Result;
use headless_chrome::{protocol::cdp::Page::CaptureScreenshotFormatOption, Browser, LaunchOptions};
use std::fs;
use std::mem::forget;

pub fn screenshot(path: String) -> Result<()> {
    let ct = fs::read_to_string(path)?;
    for line in ct.lines() {
        let symbol = line.split_whitespace().collect::<Vec<_>>()[0];
        let exchange = line.split_whitespace().rev().collect::<Vec<_>>()[0];
        if exchange != "美股" || symbol.starts_with(".") {
            continue;
        }
        let options = LaunchOptions::default_builder()
            .window_size(Some((2560, 1440)))
            .build()
            .expect("Couldn't find appropriate Chrome binary.");
        let browser = Browser::new(options)?;
        let tab = browser.new_tab()?;
        let jpeg_data = tab
            .navigate_to(
                format!("http://localhost:3000/local?symbolState=\"{}\"", symbol).as_str(),
            )?
            .wait_until_navigated()?
            .capture_screenshot(CaptureScreenshotFormatOption::Jpeg, Some(75), None, true)?;
        fs::write(format!("./data/{}.jpg", symbol), jpeg_data)?;
    }

    Ok(())
}
