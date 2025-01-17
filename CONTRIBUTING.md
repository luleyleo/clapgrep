# Contributing

A brief overview over the project, its dependencies and build instructions.

## Modules

Clapgrep consists of two modules:

- core, which contains the search logic.
- gnome, which is a Gtk + Adwaita frontend.

## Requirements (for Fedora Rawhide toolbox)

For the build system and OpenGL support:

```sh
sudo dnf install just meson appstream cargo clippy gcc libglvnd-gles
```

For the Gtk based app:

```sh
sudo dnf install just gtk4-devel libadwaita-devel
```

And you have to install [blueprint-compiler](https://jwestman.pages.gitlab.gnome.org/blueprint-compiler/setup.html).

## Compilation and Development

- Clapgrep requires up-to-date library versions (Fedora Rawhide or newest Gnome SDK)
- The [`justfile`](./justfile) provides convenient commands for compilation and development.
- `just ci` can be used to run the CI locally.
