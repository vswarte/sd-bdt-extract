# SD.bdt extract

I wrote this tool in a haze of weed and alcohol at some point because a bug in the original UXM impl pissed me off.

I made the trade-off between disk reading on the fly and copying all bytes into mem and doing stuff there because the
sd.bdt seems small enough to make the assumption that it'll fit in memory entirely and I haven't bothered cleaning it
up whatsoever.

## The UXM bug
It think that the community-standard UXM implementation uses the padded file size for the carving and doesn't strip
zeroes. This means that anything reading these files will need some crazy hack to accommodate for extra padding (which 
is what pissed me off).