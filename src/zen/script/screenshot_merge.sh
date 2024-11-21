# shellcheck disable=SC2046
img2pdf data/china_stock/*.png  --border 2cm:1cm -o china_stock-$(date +%F).pdf
# shellcheck disable=SC2046
img2pdf data/us_stock/*.png  --border 2cm:1cm -o us_stock-$(date +%F).pdf

img2pdf data/us_option/*.png  --border 2cm:1cm -o us_option-$(date +%F).pdf
LOG_LEVEL=INFO python main.py add-toc us_option-$(date +%F).pdf ./data/us_option.txt