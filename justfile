name := 'clapgrep'
appid := 'de.leopoldluley.Clapgrep'
frontend := 'clapgrep-gnome'

rootdir := ''
prefix := '/usr'

base-dir := absolute_path(clean(rootdir / prefix))

bin-src := 'target' / 'release' / name
bin-dst := base-dir / 'bin' / name

desktop := appid + '.desktop'
desktop-src := 'res' / desktop
desktop-dst := clean(rootdir / prefix) / 'share' / 'applications' / desktop

icons-src := 'res' / 'icons' / 'hicolor'
icons-dst := clean(rootdir / prefix) / 'share' / 'icons' / 'hicolor'

icon-svg-src := icons-src / 'scalable' / 'apps' / 'icon.svg'
icon-svg-dst := icons-dst / 'scalable' / 'apps' / appid + '.svg'

clean:
  cargo clean

build *args: build-translations
  cargo build --package {{frontend}} {{args}}

check *args:
  cargo clippy --all-features {{args}} -- -W clippy::pedantic

run *args: build-translations
  env RUST_BACKTRACE=full cargo run --package {{frontend}} {{args}}

gettext *args:
  xgettext \
    --from-code=UTF-8 \
    --add-comments \
    --keyword=_ \
    --keyword=C_:1c,2 \
    --language=C \
    --output=po/messages.pot \
    --files-from=po/POTFILES \
    {{args}}

add-translation language:
  msginit -l {{language}}.UTF8 -o po/{{language}}.po -i po/messages.pot

build-translations:
  cat po/LINGUAS | while read lang; do \
    mkdir -p assets/locale/$lang/LC_MESSAGES; \
    msgfmt -o assets/locale/$lang/LC_MESSAGES/{{appid}}.mo po/$lang.po; \
  done
