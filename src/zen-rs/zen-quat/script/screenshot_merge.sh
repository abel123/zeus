RUST_LOG=debug cargo run -r --bin cli -- screenshot
img2pdf data/*.jpg  --border 2cm:1cm -o out.pdf