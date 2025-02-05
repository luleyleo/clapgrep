subdir('gtk')

install_data(
  'icons' / 'hicolor' / 'scalable' / 'apps' / '@0@.svg'.format(application_id),
  install_dir: iconsdir / 'hicolor' / 'scalable' / 'apps',
)

# Desktop file
desktop_conf = configuration_data()
desktop_conf.set('icon', application_id)
desktop_file = i18n.merge_file(
  type: 'desktop',
  input: configure_file(
    input: '@0@.desktop.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: desktop_conf,
  ),
  output: '@0@.desktop'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'applications',
)

# Validate Desktop file
if desktop_file_validate.found()
  test(
    'validate-desktop',
    desktop_file_validate,
    args: [desktop_file.full_path()],
    depends: desktop_file,
  )
endif

# Appdata
appdata_conf = configuration_data()
appdata_conf.set('app-id', application_id)
appdata_conf.set('gettext-package', gettext_package)
appdata_file = i18n.merge_file(
  input: configure_file(
    input: '@0@.metainfo.xml.in.in'.format(base_id),
    output: '@BASENAME@',
    configuration: appdata_conf,
  ),
  output: '@0@.metainfo.xml'.format(application_id),
  po_dir: podir,
  install: true,
  install_dir: datadir / 'metainfo',
)

# Validate Appdata
if appstream_util.found()
  test(
    'validate-appdata',
    appstream_util,
    args: ['validate', '--nonet', appdata_file.full_path()],
    depends: appdata_file,
  )
endif

conf = configuration_data()
conf.set('APP_PATH', application_path)
clapgrep_resources = gnome.compile_resources(
  'resources',
  configure_file(
    input: 'resources.xml.in',
    output: 'resources.xml',
    configuration: conf,
  ),
  gresource_bundle: true,
  source_dir: meson.current_build_dir(),
  dependencies: [help_overlay, appdata_file],
)[0]
