# Contributing

A brief overview over the project, its dependencies and build instructions.

## Modules

Clapgrep consists of two modules:

- core, which contains the search logic.
- gnome, which is a Gtk + Adwaita frontend.

## Requirements (for Fedora toolbox)

For the build system and OpenGL support:

```sh
sudo dnf install just cargo clippy gcc libglvnd-gles python3-aiohttp python3-toml
```

For the Gtk based app:

```sh
sudo dnf install just gtk4-devel libadwaita-devel
```

And you have to install [blueprint-compiler](https://jwestman.pages.gitlab.gnome.org/blueprint-compiler/setup.html).

## Development

You can run the app with

```sh
just run
```

You can check for warnings with

```sh
just check
```

You can install the `.Devel` flatpak with

```sh
just install-flatpak
```

You can install the regular flatpak with

```sh
just release=true install-flatpak
```

And you can update the translation messages with

```sh
just gettext
```
