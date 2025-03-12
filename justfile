appid := 'de.leopoldluley.Clapgrep.Devel'

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

run: setup
  meson compile -C build && env RUST_BACKTRACE=full build/gnome/clapgrep

setup-flatpak-repos:
  flatpak remote-add --user --if-not-exists flathub https://dl.flathub.org/repo/flathub.flatpakrepo

ci: setup-flatpak-repos
  flatpak-builder --keep-build-dirs --install-deps-from=flathub --user --build-only --ccache --force-clean flatpak build-aux/{{appid}}.json
  echo Check formatting:
  ./build-aux/fun.sh cargo fmt --all -- --check --verbose
  echo Check code with Clippy:
  ./build-aux/fun.sh meson setup -Dprofile=development /run/build/clapgrep/build-ci
  ./build-aux/fun.sh meson compile -C /run/build/clapgrep/build-ci cargo-clippy

install-flatpak: setup-flatpak-repos
  flatpak-builder flatpak-build build-aux/{{appid}}.json --force-clean --install --install-deps-from=flathub --user

update-potfiles:
  rm -f po/POTFILES
  echo assets/de.leopoldluley.Clapgrep.desktop.in.in >> po/POTFILES
  echo assets/de.leopoldluley.Clapgrep.metainfo.xml.in.in >> po/POTFILES
  fd .blp gnome/src/ui >> po/POTFILES
  rg -l gettext gnome/src/ui >> po/POTFILES

update-translations: update-potfiles
  meson compile -C build clapgrep-pot
  meson compile -C build clapgrep-update-po

add-translation language:
  msginit -l {{language}}.UTF8 -o po/{{language}}.po -i po/clapgrep.pot
