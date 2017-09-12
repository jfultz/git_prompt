# git_prompt
A personal project to create a decent git prompt in Rust.

# Background
I need a replacement for the quite nice bash git prompt, which is just too slow on Windows.  Also I like to use other shells on Windows including `cmd` and `tcc`. And this has become an opportunity for me to learn Rust.

# License
[MIT License](LICENSE).  This project uses [libgit2](https://libgit2.github.com/) via [git2-rs](https://github.com/alexcrichton/git2-rs) (and also imports many other dependencies of the git2 crate which I won't specifically call out).
