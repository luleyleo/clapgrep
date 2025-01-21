help_overlay = custom_target(
  'help-overlay',
  input: ['help-overlay.blp'],
  output: ['help-overlay.ui'],
  command: [
    blueprint_compiler,
    'compile',
    '--output', 'assets/resources/help-overlay.ui',
    '@INPUT@',
  ],
)

clapgrep_resources = gnome.compile_resources(
  'resources',
  'resources.xml',
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  dependencies: [help_overlay],
)[0]
