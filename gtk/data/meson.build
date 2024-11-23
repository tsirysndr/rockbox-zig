conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('PREFIX', prefix)
conf.set('VERSION', meson.project_version())
conf.set('pkgdatadir', pkgdatadir)


gnome.compile_resources(
  'rockbox',
  configure_file(
    input: 'rockbox.gresource.xml.in',
    output: 'rockbox.gresource.xml',
    configuration: conf,
  ),
  gresource_bundle: true,
  install: true,
  install_dir: pkgdatadir,
)

desktop_file = i18n.merge_file(
  input: configure_file(
    input: 'mg.tsirysndr.Rockbox.desktop.in',
    output: app_id + '.desktop.in',
    configuration: conf,
  ),
  output: app_id + '.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'applications'),
)


subdir('icons')