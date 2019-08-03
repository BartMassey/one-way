# One Way
Bart Massey (ThePO8)

*One Way* is a
[roguelike](https://en.wikipedia.org/wiki/Roguelike) written
in [Rust](https://www.rust-lang.org) for the August 2019
[GMTK Game Jam](https://itch.io/jam/gmtk-2019).

The theme of GMTK 2019 is "Only One". My son Benjamin Massey
(Bean), who is brilliant, tossed out about 20 great ideas
around this theme. I liked one of them so much I decided to
run with it.

*One Way* is a multiplayer (yes, I know, but wait)
pure-ASCII roguelike
[telnet](https://en.wikipedia.org/wiki/Telnet)
"[Door Game](https://en.wikipedia.org/wiki/Category:Door_games)".
The dungeon has only one dimension — you start at the left
end, and there is only one way out: through the exit door
(there is only one exit door) at the right end. There is
only one way to get through that door: defeat the Boss (the
only one in the game) that is guarding it, and take from him
the only key that unlocks your exit.

## Acknowledgments

Thanks to Bean for the excellent game idea.

This game would not have been possible in the given time
without the excellent Rust
[`telnet-rs`](https://github.com/SLMT/telnet-rs) crate.  It
was intended for clients, but it turns out it works great
for servers too!

## Source Code and License

Please see http://gitlab.com/BartMassey/one-way for the
complete repository.

This program is licensed under the "GPL version 3 or
later". Please see the file `LICENSE` in this distribution
for license terms.
