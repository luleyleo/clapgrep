help_overlay = custom_target(
  'help-overlay',
  input: ['help-overlay.blp'],
  output: ['help-overlay.ui'],
  command: [
    blueprint_compiler,
    'compile',
    '--output', 'assets/gtk/help-overlay.ui',
    '@INPUT@',
  ],
)
