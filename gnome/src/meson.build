build_conf = configuration_data()
build_conf.set_quoted('APP_ID', application_id)
build_conf.set_quoted('APP_PATH', application_path)
build_conf.set_quoted('APP_VERSION', meson.project_version())
build_conf.set_quoted('GRESOURCES_BUNDLE', clapgrep_resources.full_path())

configure_file(input: 'build.rs.in', output: 'build.rs', configuration: build_conf)

run_command(
  'cp',
  join_paths(meson.project_build_root(), 'gnome', 'src', 'build.rs'),
  join_paths(meson.project_source_root(), 'gnome', 'src', 'build.rs'),
  check: true,
)
