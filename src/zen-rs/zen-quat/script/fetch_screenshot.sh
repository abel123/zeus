
cat ./script/watchlist.txt | grep 美股 | grep -v '^\.' | while read line 
do
    echo $line
done

curl --request POST http://localhost:3001/forms/chromium/screenshot/url \
--form url='http://localhost:3000/local?symbolState="TSLA"' \
--form format=jpeg \
--form quality=100 \
--form waitDelay=5s \
--form width=3840 \
--form height=2160 \
-o my.jpeg

