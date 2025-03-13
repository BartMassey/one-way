# Finding One Way Out
Bart Massey 2025-03-13

*One Way Out* is a
[roguelike](https://en.wikipedia.org/wiki/Roguelike)
[telnet](https://en.wikipedia.org/wiki/Telnet) ["door
game"](https://en.wikipedia.org/wiki/Category:Door_games) written
in [Rust](https://www.rust-lang.org) for the August 2019
[GMTK Game Jam](https://itch.io/jam/gmtk-2019).


## Topics

Today we'll talk about

* *One Way Out* (OWO)
* Game Jams
* Telnet, Door Games and Roguelikes
* The OWO Rust code

Then we'll think about extending OWO if time permits.

## One Way Out

My Game Jam game from 2019. On your computer, say

    telnet myzulip.bart-massey.com

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

My game just uses bare "carriage return". It can get away
with this because reasons you've seen.
