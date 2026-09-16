@echo off
REM OpenComp — full plan demo (Phases 1-6)
cd /d "%~dp0"
echo === OpenComp demo — all phases ===
opencomp.exe render demo.toml -f 0 -o demo_f000.png
opencomp.exe render demo.toml -f 60 -o demo_f060.png
opencomp.exe render demo.toml -f 120 -o demo_f120.png
echo.
echo === Render full animation to MP4 (Phase 6) ===
opencomp.exe render demo.toml --all -o demo.mp4
echo.
echo Files produced:
dir /B *.png *.mp4
echo.
echo Done! Open demo_f000.png / demo_f060.png / demo_f120.png (frames)
echo and demo.mp4 (full 5-second animation).
pause