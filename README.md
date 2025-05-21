**Brick Shell** is a small shell adapted from [Bubble Shell](https://github.com/JoshMcguigan/bubble-shell/) written in Rust

*Note: This shell has only been tested un Ubuntu and MacOS Sequoia 15.5, if you are running another OS you might have to make adjustments*
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

Once you have Rust installed just run `cargo build` to build the executable.

To use this shell, you need to do the following things: 

1. run `bash move_shell.bash` to copy the executable to `/bin/`

2. run `chsh` and enter `/bin/brick_shell` as your new shell path

3. go into `/etc/passwd` and change the line that looks like `{user}:x:1000:1000:{user}:/home/{user}:/bin/{your active shell}`
to `{user}:x:1000:1000:{user}:/home/{user}:/bin/brick_shell`

(`{user}` denotes the currently active username)

On Ubuntu if you want the shell to startup when you click your terminal do the following: 
Right-click on the terminal icon -> Preferences -> Select a profile (your start profile should just be `Unnamed`) -> Command

Here check `Run command as login shell` and `Run custom command instead of my shell` 

as the command you enter `/bin/brick_shell`

---
## MacOS

To use this shell on Mac, there are a few more steps to follow.

1. Change line 6 of `src/commons.rs` from `"/home/".to_owned() + get_user_by_uid(get_current_uid()).unwrap().name().to_str().unwrap()` to `"/Users/<username here>/.to_owned()`

2. Run `cargo bulild` to compile your shell.

4. Now you do `nano /etc/shells` and add `<your home directory here>/Brick-Lang/target/debug/brick_shell`.

3. When in the MacOS Terminal, press `Command + ,`

5. Now tick `Command` instead of `Default shell` and paste `<your home directory here>/Brick-Lang/target/debug/brick_shell`.
