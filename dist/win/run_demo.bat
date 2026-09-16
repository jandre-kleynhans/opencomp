@echo off
REM OpenComp Windows demo runner — renders 3 frames of the animated demo.
REM Usage: double-click or run from cmd. PNGs land in the same folder.
cd /d "%~dp0"

echo === OpenComp Windows demo ===
echo Rendering demo.toml frame 0/60/120...

opencomp.exe render demo.toml -f 0 -o demo_f000.png
opencomp.exe render demo.toml -f 60 -o demo_f060.png
opencomp.exe render demo.toml -f 120 -o demo_f120.png

echo.
echo Done. Open demo_f000.png / demo_f060.png / demo_f120.png to see the animation.
pause