#!/usr/bin/env bash
set -euxo pipefail

#cargo web build --release --target wasm32-unknown-unknown -p oxi8_kiss3d
cargo web build --release --target wasm32-unknown-unknown -p oxi8_quicksilver
cargo build --release

# for dev, in subdir:
# cargo web start --target wasm32-unknown-unknown --open --auto-reload

# build games.html
cat > oxi8_quicksilver/static/games.html <<EOF
<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8" />
    <title>oxi8 rom listing</title>
    <meta http-equiv="X-UA-Compatible" content="IE=edge" />
    <meta content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=1" name="viewport" />
</head>
<body>
<pre>
chip8 had hexadecimal keyboard like on the right, here is the mapping:
1234  --->  123C
QWER  --->  456D
ASDF  --->  789E
ZXCV  --->  A0BF

Additional keys specific to oxi8:
Enter     ---> Reset game
Backspace ---> Back to game listing
Space     ---> Pause game
+/=       ---> Increase CPU Hz by 10
_/-       ---> Decrease CPU Hz by 10
0         ---> Set CPU Hz to Chip-8 default of 500hz
9         ---> Set CPU Hz to SChip default of 1000hz
</pre>
<a href="https://github.com/moparisthebest/oxi8">oxi8 git repo here</a><br/>
Click a game to play in your browser:
<ul>
EOF
for game in $(find resources/CHIP8/GAMES/ -type f ! -name '*.*')
do
anchor=$(base64 -w0 < "$game")
name=$(basename "$game")
cat >> oxi8_quicksilver/static/games.html <<EOF
<li><a href="./#$anchor">$name</a></li>
EOF
done
echo '<li>SCHIP GAMES:</li>' >> oxi8_quicksilver/static/games.html
for game in $(find resources/CHIP8/SGAMES/ -type f ! -name '*.*')
do
anchor=$(base64 -w0 < "$game")
name=$(basename "$game")
cat >> oxi8_quicksilver/static/games.html <<EOF
<li><a href="./#$anchor">$name</a></li>
EOF
done

echo '<li>OCTO GAMES, some may need to run significantly faster, press 9 then hold +:</li>' >> oxi8_quicksilver/static/games.html
for game in $(find resources/Octo/examples/ -type f ! -name '*.*')
do
# skip xo games for now
echo "$game" | grep -i 'xo' &>/dev/null && continue || true
anchor=$(base64 -w0 < "$game")
name=$(basename "$game")
cat >> oxi8_quicksilver/static/games.html <<EOF
<li><a href="./#$anchor">$name</a></li>
EOF
done

cat >> oxi8_quicksilver/static/games.html <<EOF
</ul>
</body>
</html>
EOF

rm -rf target/oxi8_quicksilver*
mkdir target/oxi8_quicksilver_web
cp oxi8_quicksilver/static/* target/wasm32-unknown-unknown/release/oxi8_quicksilver.* target/oxi8_quicksilver_web/
(cd target/oxi8_quicksilver_web/ && zip -r ../oxi8_quicksilver.zip .)

# rsync -avz ./target/oxi8_quicksilver_web/ vps:/home/mopar/htdocs/moparisthebest.com/oxi8/ --delete
