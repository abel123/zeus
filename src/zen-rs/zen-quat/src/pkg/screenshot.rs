use anyhow::Result;
use headless_chrome::{protocol::cdp::Page::CaptureScreenshotFormatOption, Browser, LaunchOptions};
use std::fs;
use std::mem::forget;
use tracing::{debug, error};
use tws_rs::contracts::Contract;

pub fn parse_contract(line: &str) -> Option<Contract> {
    let symbol = line.split_whitespace().collect::<Vec<_>>()[0];
    let exchange = line.split_whitespace().rev().collect::<Vec<_>>()[0];

    if exchange == "美股" {
        if symbol.starts_with(".") {
            return None;
        }
        return Some(Contract::auto_stock(symbol));
    } else if exchange == "沪深" {
        if !['0', '6', '3'].contains(&symbol.to_string().chars().next().unwrap_or(' ')) {
            return None;
        }
        if ['0', '3'].contains(&symbol.to_string().chars().next().unwrap_or(' ')) {
            return Some(Contract::auto_stock(format!("SZSE:{}", symbol).as_str()));
        }
        return Some(Contract::auto_stock(format!("SSE:{}", symbol).as_str()));
    }

    None
}
pub fn screenshot(path: String, outdir: String) -> Result<()> {
    let ct = fs::read_to_string(path)?;
    let options = LaunchOptions::default_builder()
        .enable_gpu(false)
        .window_size(Some((3840, 2160)))
        .build()
        .expect("Couldn't find appropriate Chrome binary.");
    let browser = Browser::new(options)?;

    for (idx, line) in ct.lines().enumerate() {
        let contract = parse_contract(line);
        if contract.is_none() {
            continue;
        }
        let contract = contract.unwrap();

        let res = || -> Result<()> {
            debug!("crawling {}-{}", contract.exchange, contract.symbol);

            let tab = browser.new_tab()?;
            let jpeg_data = tab
                .navigate_to(
                    format!(
                        "http://localhost:3000/local?symbolState=\"{}\"",
                        if contract.currency == "CNH" {
                            if contract.symbol.starts_with("6") {
                                format!("SSE:{}", contract.symbol)
                            } else {
                                format!("SZSE:{}", contract.symbol)
                            }
                        } else {
                            contract.symbol.clone()
                        }
                    )
                    .as_str(),
                )?
                .wait_until_navigated()?
                .capture_screenshot(CaptureScreenshotFormatOption::Jpeg, Some(75), None, true)?;
            fs::write(
                format!(
                    "./{}/{:03}-{}.jpg",
                    outdir.trim_end_matches("/"),
                    idx,
                    contract.symbol.clone()
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
