# shellcheck disable=SC2046
img2pdf data/china_stock/*.jpg  --border 2cm:1cm -o china_stock-$(date +%F).pdf
# shellcheck disable=SC2046
img2pdf data/us_stock/*.jpg  --border 2cm:1cm -o us_stock-$(date +%F).pdf
