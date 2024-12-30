conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('PREFIX', prefix)
conf.set('VERSION', meson.project_version())
conf.set('NAME', name)
conf.set('PKGNAME', app_id)
conf.set('pkgdatadir', pkgdatadir)

blueprints = custom_target(
  'blueprints',
  input: files(
    'gtk/album_details.blp',
    'gtk/album_tracks.blp',
    'gtk/albums.blp',
    'gtk/artist.blp',
    'gtk/artist_details.blp',
    'gtk/artists.blp',
    'gtk/current_playlist.blp',
    'gtk/file.blp',
    'gtk/files.blp',
    'gtk/help-overlay.blp',
    'gtk/likes.blp',
    'gtk/media_controls.blp',
    'gtk/playlist.blp',
    'gtk/playlist_folder.blp',
    'gtk/playlists.blp',
    'gtk/playlist_details.blp',
    'gtk/preferences.blp',
    'gtk/search.blp',
    'gtk/song.blp',
    'gtk/songs.blp',
    'gtk/window.blp',
  ),
  output: '.',
  command: [
    find_program('blueprint-compiler'),
    'batch-compile',
    '@OUTPUT@',
    '@CURRENT_SOURCE_DIR@',
    '@INPUT@',
  ]
)

appstream_file = i18n.merge_file(
  input: configure_file(
    input: 'io.github.tsirysndr.Rockbox.metainfo.xml.in.in',
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
    input: 'io.github.tsirysndr.Rockbox.desktop.in.in',
    output: 'io.github.tsirysndr.Rockbox.desktop.in',
    configuration: conf,
  ),
  output: 'io.github.tsirysndr.Rockbox.desktop',
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
  dependencies: [appstream_file, blueprints],
)

subdir('icons')
