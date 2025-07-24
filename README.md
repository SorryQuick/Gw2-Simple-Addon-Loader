# Gw2-Simple-Addon-Loader
A simple addon loader that can load any exe or dll with the game.
Also works as a workaround to have multiple programs running in a single steam container at the same time.

# How to Use

This expects this exe to be in <Gw2 path containing Gw2-64.exe>/addons/<subfolder name of your choice>/

Example path: 
~/.steam/steam/steamapps/common/Guild Wars 2/addons/overlay/Gw2-Simple-Addon-Loader.exe

In this folder, create two files: dlls.txt and exes.txt 
All DLL paths found in the dlls file will be loaded once the game is launched.
All EXE paths found in the exes file will be loaded once the game is launched.
The format of the file is simply one absolute path OR relative path per line.

If you use WINE, you need to use the wine path (eg. V:/home/user....) or the relative path.

To make use of this launcher, simply replace your steam/lutris path that usually points to Gw2-64.exe and make it point to this launcher instead.
