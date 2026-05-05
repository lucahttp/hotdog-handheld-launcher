# Panchito Project

the idea is to have a lightweight launcher for windows handheld gaming devices. dont waste a megabyte of ram or time of the cpu. It should be fast, responsive, and easy to use.

using some features from the following projects:
- zed GPUI for ui (https://github.com/zed-industries/zed) <Uses rust and it baremetal as possible which are good for performance>
- components from `gpui-component` for a nice metro style (https://github.com/Longbridge/gpui-component)
- https://github.com/irv77/Xbox360UI for UI reference

add some dont disturb features to disable all non-gaming features from windows
https://github.com/hellzerg/optimizer
https://github.com/ChrisTitusTech/winutil
https://github.com/memstechtips/Winhance

we dont care about files, indexing,copilot, telemetry, office, onedrive, and a ton of other really useful features for a day-to-day computer use. but for a gaming device all of them are useless and adding roadblocks in the way to play games. 

we want to make it easy to play games on a windows handheld gaming device. 

No Login/Updates/Cloud features.

only games and easy access to settings. with a nice UI. 




AI Skills that I have used


https://github.com/juliusbrussee/caveman#install
https://github.com/actionbook/rust-skills
https://github.com/longbridge/gpui-component/tree/main/.claude/skills



# Apply optimizations (requiere admin)
.\optimizations\gaming-mode.ps1

# Revertir
.\optimizations\gaming-mode.ps1 -Rollback

# Vista previa sin cambios
.\optimizations\gaming-mode.ps1 -DryRun