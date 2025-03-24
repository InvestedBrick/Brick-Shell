**Brick Shell** is a small shell adapted from [Bubble Shell](https://github.com/JoshMcguigan/bubble-shell/) written in Rust

*Note: This shell has only been tested un Ubuntu, if you are running another OS you might have to make adjustments*
## Features

Brick Shell currently supports the following features:

- Chaining commands through pipes
- A custom `ls` implementation
- Command history, which can be cleared using `clear-history`
- Autocompletion using `TAB` for files, directories and commonly used commands

    *Note: Commands get added to a list and are available for autocompletion upon restart of the shell*
- Hinting what the shell will autocomplete to
- aliases using `alias [-p / -t] [name] "alias"` Where -p indicates permanent aliases and -t only aliases for this session.
- Chaining commands by adding ` && ` betweem them

You can restart the shell and by that also reload aliases etc by using `restart`
## Compiling && Running
If you don't have Rust installed, you can get it [here](https://www.rust-lang.org/tools/install)

Once you have Rust installed just run `cargo run` run compile and run.

You can also run the executable with the flag `--in-shell`, which then will not create a new window but rather just spawn the shell in your current terminal
(useful for anyone using this on a CLI distro)

If you want to replace bash with this better shell but don't want to risk locking yourself out of your user because this shell does not support logins... (There might have been an incident)

... You can just add the following to the end of your `~/.bashrc` file

```
<path to source dir>/target/debug/brick_shell
sleep 0.2
exit
```

*Note: I would only recomend this if you are launching a new window of the shell, else this will just exit bash and you have no shell*