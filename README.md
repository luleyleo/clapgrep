# Clapgrep

One app to search through all your files, powered by ripgrep.

![screenshot of the app](assets/screenshot-1.png)

There are currently two frontends, one written with Gtk and one with Iced. The Gtk one is much better right now.

## Modules

Clapgrep consists of three modules:

- core, which contains the search logic.
- gnome, which is a Gtk + Adwaita frontend.
- cosmic, which is a Iced + libcosmic based frontend.
