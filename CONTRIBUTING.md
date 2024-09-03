# Contributing

A brief overview of dependencies and build instructions.

## Requirements (for Fedora toolbox)

For the build system and OpenGL support:

```sh
sudo dnf install just cargo clippy gcc libglvnd-gles
```

For the Gtk based app:

```sh
sudo dnf install just gtk4-devel libadwaita-devel
```

For the Iced based app:

```sh
sudo dnf install cmake expat-devel fontconfig-devel freetype-devel libxkbcommon-devel
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
