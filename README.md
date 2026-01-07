# lazycp - copying for the terminally ill

> [!NOTE]
> WORK IN PROGRESS! Currently, only basic file copying/pasting is implemented.

If you have a few too many panes, terminal windows, tabs, muxers, commandlines, and various other
shell doodads open at the same time, and just want to get one damn file from the working directory
of one to the working directory of another, then *lazycp* is the app for you.

With a few aliases, typing out the fully qualified path to a directory you already have open in one
of your multitude muxed terminal panes becomes as simple as:

```sh
# In one of your shells
lcp file1.txt # lcp is aliased to `lcp copy`

# In another one of your shells - lp is aliased to `lcp paste`
lp # lp is aliased to `lcp paste`
```

And so it is done.

## How it works

Dead simple: any copy or move command is saved to a copy/move history file and thus referenced by
the paste command.

## Copying lazily over remote connections

*lazycp* also works over remote connections via SSH (kind of)! If *lazycp* is installed on both host
and client machines, the program is capable of reading the remote machine's copy/move history file
and pasting whatever entry you specify to your local machine.

So, if you have two panes open, one of them being a remote connection won't stop your laziness!

```sh
# On one machine
lcp remotefile.txt
# On another machine
lp -h remote@192.168.1.100
```

You can imagine that setting up aliases for specific commonly used machines, alongside ssh-agent,
would make the above process decently seamless.

```sh
# Paste from specific machine
alias lpr="lp -h remote@192.168.1.86"

# Paste from commonly used subnet with specified address (192.168.86.x)
lps () {
    lp -h "remote@192.168.86.$1"
}

lpr # paste from remote
lps 100 # paste from 192.168.86.100
```

## Install

Currently only for nerds who know how to build software.

```sh
cargo install --path .
```

## License

> lazycp - the easiest way to copy for people with too many terminals open  
> Copyright (C) 2026 Artemis Rosman
>
> This program is free software: you can redistribute it and/or modify
> it under the terms of the GNU General Public License as published by
> the Free Software Foundation, either version 3 of the License, or
> (at your option) any later version.
>
> This program is distributed in the hope that it will be useful,
> but WITHOUT ANY WARRANTY; without even the implied warranty of
> MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
> GNU General Public License for more details.
>
> You should have received a copy of the GNU General Public License
> along with this program.  If not, see <https://www.gnu.org/licenses/>.
