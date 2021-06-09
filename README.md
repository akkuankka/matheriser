# matheriser
*Evaluates maths expressions of increasing complexity*

## Features
* Algebraically handles rational numbers, surds, and irrational constants like pi
* Avoids using floating point numbers as much as possible, because they're inherently imprecise
* Currently only a terminal interface, with scope for a GUI later down the line

## Using matheriser
Honestly I wouldn't at this stage, the terminal interface is pretty clunky, until we have a GUI/TUI and support for trig functions there's really no ergonomics advantage over any other computer calculator.

There is no intent to put this on crates.io for a good while yet, because it's still quite useless ~~and also I don't know how it works~~, so the only way to get your hands on some steamy fresh matheriser is to clone this repository (maybe the indev branch) and `cargo build` it. As of yet I do not know if `cargo install` will work, I suspect it won't because there are some localisation files that need to be bundled away and cargo doesn't seem to like doing that 

## Roadmap to `0.2.0` 
- [ ] Implement syntax for non-typable functions (i.e. log, roots)
- [ ] Implement trigonometric functions
- [ ] Improve the prompt of the terminal interface

## Where to from there:
- [ ] A TUI frontend
- [ ] A GUI frontend (not electron so currently it's looking like [Druid](https://github.com/linebender/druid)?)
- [ ] *LINEAR ALGE* ***BRUH***
- [ ] *** Learn how linear algebra works *** -> Profit???

