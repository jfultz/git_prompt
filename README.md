# git_prompt
A personal project to create a decent git prompt in Rust.

# Background
I need a replacement for the quite nice bash git prompt, which is just too slow on Windows.  Also I like to use other shells on Windows including `cmd` and `tcc`. And this has become an opportunity for me to learn Rust.

# Usage

### bash
E.g., here's what I'm using right now...
```
# Setup variables for the binary and the non-git portions of the prompt
export GIT_PROMPT_BINARY=<path to git_prompt binary>
export GIT_PROMPT_START="${COLOR_GRAY}\T ${COLOR_BLUE}\h:${COLOR_GREEN}\w${COLOR_NONE} "
export GIT_PROMPT_END="\n>"

# This is executed by bash before each prompt, and it resets PS1 to
# be updated based upon the current state of the shell, assuming
# GIT_PROMPT_BINARY is valid.
set_prompt() {
  if [[ -e $GIT_PROMPT_BINARY ]]; then
    export PS1="${GIT_PROMPT_START}`${GIT_PROMPT_BINARY}`${GIT_PROMPT_END}"
  fi
}
export PROMPT_COMMAND='set_prompt'
```

### TCC
E.g., here's what I'm using right now...
```
SET GIT_PROMPT_BINARY=<path to git_prompt binary>
prompt `$e[1;30m$M $e[0;34m%COMPUTERNAME $e[0;32m$P$e[0m %@execstr[%GIT_PROMPT_BINARY]$_$g`
```

# License
[MIT License](LICENSE).  This project uses [libgit2](https://libgit2.github.com/) via [git2-rs](https://github.com/alexcrichton/git2-rs) (and also imports many other dependencies of the git2 crate which I won't specifically call out).
