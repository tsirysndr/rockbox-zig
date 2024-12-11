conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('PREFIX', prefix)
conf.set('VERSION', meson.project_version())
conf.set('NAME', name)
conf.set('PKGNAME', app_id)
conf.set('pkgdatadir', pkgdatadir)

appstream_file = i18n.merge_file(
  input: configure_file(
    input: 'mg.tsirysndr.Rockbox.metainfo.xml.in.in',
    output: app_id + '.metainfo.xml.in',
    configuration: conf,
  ),
  output: app_id + '.metainfo.xml',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'metainfo'),
)

desktop_file = i18n.merge_file(
  input: configure_file(
    input: 'mg.tsirysndr.Rockbox.desktop.in',
    output: 'rockbox.desktop.in',
    configuration: conf,
  ),
  output: 'rockbox.desktop',
  type: 'desktop',
  po_dir: '../po',
  install: true,
  install_dir: join_paths(get_option('datadir'), 'applications'),
)

appstreamcli = find_program('appstreamcli', required: false)
if appstreamcli.found()
  test(
    'Validate appstream file',
    appstreamcli,
    args: ['validate', '--no-net', '--explain', appstream_file],
    workdir: meson.current_build_dir(),
  )
endif

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
  dependencies: appstream_file,
)

subdir('icons')
