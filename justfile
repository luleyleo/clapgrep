appid := 'de.leopoldluley.Clapgrep'

default:
  just --list

clean:
  cargo clean
  rm -rf build
  rm -rf flatpak
  rm -rf flatpak-build
  rm -rf .flatpak-builder

setup:
  [ -d build ] || meson setup -Dprofile=development build

check: setup
  meson compile -C build cargo-clippy

build: setup
  meson compile -C build

test: setup
  meson test -C build

run *args: setup
  meson compile -C build && env RUST_BACKTRACE=full build/gnome/clapgrep {{args}}

setup-flatpak-repos:
  flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

ci: setup-flatpak-repos
  flatpak-builder --keep-build-dirs --install-deps-from=flathub --user --build-only --ccache --force-clean flatpak build-aux/{{appid}}.CI.json
  ./build-aux/fun.sh meson setup -Dprofile=development /run/build/clapgrep/build-ci
  echo Check formatting:
  ./build-aux/fun.sh cargo fmt --all -- --check --verbose
  echo Check code with Clippy:
  ./build-aux/fun.sh meson compile -C /run/build/clapgrep/build-ci cargo-clippy

install-flatpak: setup-flatpak-repos
  flatpak-builder flatpak-build build-aux/{{appid}}.Devel.json --force-clean --install --install-deps-from=flathub --user

update-potfiles:
  rm -f locale/POTFILES
  echo assets/gtk/help-overlay.blp >> locale/POTFILES
  echo assets/de.leopoldluley.Clapgrep.desktop.in.in >> locale/POTFILES
  echo assets/de.leopoldluley.Clapgrep.metainfo.xml.in.in >> locale/POTFILES
  fd .blp gnome/src/ui >> locale/POTFILES
  rg -l gettext gnome/src/ui >> locale/POTFILES

update-translations: update-potfiles
  meson compile -C build {{appid}}-pot
  meson compile -C build {{appid}}-update-po

add-translation language:
  msginit -l {{language}}.UTF8 -o po/{{language}}.po -i po/clapgrep.pot
