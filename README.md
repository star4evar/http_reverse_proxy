# http_reverse_proxy
a practice project , implement http reverse proxy with rust, following https://www.youtube.com/watch?v=FcHYQMRfGWw&amp;list=WL&amp;index=1


following links may be helpful understanding the code:

https://www.fpcomplete.com/blog/captures-closures-async/


run this project by command: cargo run

then visit localhost:3000, this will redirect you to https://www.snoyman.com

visit localhost:3000/status will show how many requests has been proxied.
