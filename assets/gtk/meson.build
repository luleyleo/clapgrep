shortcuts_dialog = custom_target(
  'shortcuts-dialog',
  input: ['shortcuts-dialog.blp'],
  output: ['shortcuts-dialog.ui'],
  command: [
    blueprint_compiler,
    'compile',
    '--output', 'assets/gtk/shortcuts-dialog.ui',
    '@INPUT@',
  ],
)