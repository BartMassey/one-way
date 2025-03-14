# Finding One Way Out
Bart Massey 2025-03-13

*One Way Out* <https://github.com/BartMassey/one-way-out> is
a [roguelike](https://en.wikipedia.org/wiki/Roguelike)
[telnet](https://en.wikipedia.org/wiki/Telnet) ["door
game"](https://en.wikipedia.org/wiki/Category:Door_games)
written in [Rust](https://www.rust-lang.org) for the August
2019 [GMTK Game Jam](https://itch.io/jam/gmtk-2019).


## Topics

Today we'll talk about

* *One Way Out* (OWO)
* Game Jams
* Telnet, Door Games and Roguelikes
* The OWO Rust code

Then we'll think about extending OWO if time permits.

## One Way Out

My Game Jam game from 2019. On your computer, say

```
telnet fob4.po8.org 10001
```

* Use `h` and `l` to move, `.` to rest, `q` to quit.

* Go right to reach the door and escape.

* Health is shared.

## Game Jams

A *Game Jam* is a limited-time contest/not-contest game
making activity. Non-computer game jams are totally a thing,
but my 2019 jam was computer games on `itch.io` via Game
Makers ToolKit.

* Must code "from scratch"
* Can bring framework, assets
* Time is of the essence

## Telnet

Telnet is perhaps the oldest way of interacting with another
computer across the Internet. A Telnet *client* connects to
a Telnet *server* (though quite symmetric) via a TCP stream.

By default, Telnet is just basically line-oriented TCP
interaction. But an in-band protocol can be used to
negotiate options.

For my game server, I wanted character-at-a-time, no local
echo, and to know the client terminal width.

## Telnet and "Character Graphics"

Telnet is run from some "terminal emulator" that has "escape
codes" to do things like "clear the screen", "change the
text color", etc.

My game just uses bare "carriage return" and then
re-renders. It can get away with this because reasons you've
seen.

## Door Games and Roguelikes

An old-school BBS typically offered games you could play
that were external to the BBS. I wanted to reproduce that
experience on modern hardware.

A Roguelike was tradtionally character-graphics plus
procedural generation plus no extra lives. This is the
only 1D Roguelike I know of.

## Writing One Way Out

* Took about 20 hours over 1.5 days. I was less comfortable
  in Rust then, but the code was still OK.

* Burned way too much time on ANSI graphics I didn't use.

* Code was uncommented. There was a lot of *ad-hocery*.

* Worked, but not a fun game because too few features.

## Multiplayer Was Interesting

* No async/await (for `telnet` crate at least). So 
  careful multithreading.

* Architecture:

  * Game world shared by `Arc` `Mutex`. (`Arc` maybe
    unnecessary.)

  * Game state updated by acting client after action.

  * Game state reset when player count goes to zero.

## Code Walkthrough

Let's look at the source and maybe hack it up a bit.

## Conclusions

* Sometimes a good game idea is killed by a bad
  prototype. This.
  
* Rust was not a bad choice here. Language maybe matters
  little.
  
* There still could be fun to be had with this.
