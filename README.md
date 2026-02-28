<p align="center">
  <img width="256" height=auto src="logo.png">
  <br/>
  <img src="https://img.shields.io/github/contributors/bancedev/forge" alt="contributors">
  <img src="https://img.shields.io/github/license/bancedev/forge" alt="license">
  <img src="https://img.shields.io/github/forks/bancedev/forge" alt="forks">
</p>

---

# Forge

Forge is a from-source package manager that allows for the seamless tracking, installing, and updating of packages build from git repositories. Forge is entirely independent of any upstream repository project you only need a git repo with the source code.

Forge is mostly designed to be used alongside an operating system with existing package managers like Arch Linux or Debian Linux with forge acting as a simple tool to supplement packages that don't exist or give user freedom to use from-source compilation to optimize performance on certain applications. At this point in time forge has no concept of a dependency tree for the packages it tracks but it may at some point in the future.

---

## Compiling/Installation

Forge uses [just](https://github.com/casey/just) to simplify build and install.
```
just
sudo just install
```

Then you can bootstrap forge to track itself by doing.
```
sudo forge add https://git.bance.dev/forge.git
```
