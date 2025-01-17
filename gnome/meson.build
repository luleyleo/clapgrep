cargo_env = environment()
if not get_option('extern_cargo_env')
  cargo_env.set('CARGO_HOME', meson.project_build_root() / 'cargo-home')
endif
cargo_env.set('APP_ID', application_id)

cargo_options = []
cargo_options += ['--manifest-path', meson.project_source_root() / 'Cargo.toml']
cargo_options += ['--target-dir', meson.project_build_root()]

if get_option('profile') == 'default'
  cargo_options += [ '--release' ]
  rust_target = 'release'
  message('Building in release mode')
else
  rust_target = 'debug'
  message('Building in debug mode')
endif

custom_target(
  'cargo-build',
  build_by_default: true,
  build_always_stale: true,
  output: meson.project_name(),
  console: true,
  install: true,
  install_dir: bindir,
  env: cargo_env,
  command: [
    cargo, 'build',
    cargo_options,
    '&&',
    'cp', rust_target / 'clapgrep-gnome', '@OUTPUT@',
  ]
)

run_target(
  'cargo-clippy',
  env: cargo_env,
  command: [cargo, 'clippy', cargo_options, '--workspace']
)

test(
  'cargo-test',
  cargo,
  env: cargo_env,
  args: ['test', cargo_options, '--workspace'],
  is_parallel: false,
)
