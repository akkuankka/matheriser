# matheriser
*Evaluates maths expressions of increasing complexity*

## Features
* Algebraically handles rational numbers, surds, and irrational constants like pi
* Avoids using floating point numbers as much as possible, because they're inherently imprecise
* Currently only a terminal interface, with scope for a GUI later down the line

## Installing
Currently the way to get a matheriser is to build it from source by cloning the github repository

You need a rust installation to build it, so if you don't have one (If you don't know what that is then you don't have one), follow the directions [here](https://rustup.rs/)

### Linux, Mac, WSL (BSD?)

In a terminal, go to a directory you don't mind a mess, and `git clone` [this repository](https://github.com/akkuankka/matheriser) like so:
```bash
$ git clone https://github.com/akkuankka/matheriser 
```
Then, depending on your OS, run the appropriate install script like so:
```bash
$ install-linux.sh # on Linux
$ install-mac.sh # on Mac
```

If you get an error saying "file install-[whatever].sh is not executable by this user", you can run
```bash
$ chmod +x ./*.sh
```
which will make all of the install scripts executable.

### Windows

If you don't have git for windows you can get it [here](https://gitforwindows.org/).
Then open up a CMD window, a Powershell window, or a Git Bash window (Comes with git for windows), or Windows Terminal if you have it, type the command
``` bash
git clone https://github.com/akkuankka/matheriser
```
Open wherever you were in the command prompt in File Explorer and copy the assets folder.
Press `WindowsKey + R`, and in the box that comes up enter `%appdata%`, then press enter, this should bring up a folder,
From there, go into the folder `Local` and then create a folder matheriser, then paste the assets folder you copied earlier there.

You should now be able to run matheriser from a terminal by typing in `matheriser` at the command prompt

## Where to next:
- [ ] Make a package
- [ ] A TUI frontend
- [ ] A GUI frontend (not electron so currently it's looking like [Druid](https://github.com/linebender/druid)?)
- [ ] *LINEAR ALGE* ***BRUH***
- [ ] *** Learn how linear algebra works *** -> Profit???

