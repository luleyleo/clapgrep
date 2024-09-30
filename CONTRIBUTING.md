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

## Development

You can run the app with

```sh
just run
```

You can check for warnings with

```sh
just check
```

And you can update the translation messages with

```sh
just gettext
```
