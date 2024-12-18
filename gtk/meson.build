project('rockbox-gtk', 'rust',
          version: '0.1.0',
    meson_version: '>= 0.59.0',
          license: 'GPL-3.0-or-later',
)

gnome = import('gnome')
i18n = import('i18n')

pkgdatadir = join_paths(get_option('prefix'), get_option('datadir'), meson.project_name())

# Dependencies

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1', version: '>=1.6.beta')

# Initialize project variables
name       = 'Rockbox'
base_id    = 'io.github.tsirysndr.Rockbox'
path_id    = '/io/github/tsirysndr/Rockbox'
app_id     = base_id
vcs_tag    = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
datadir    = join_paths(get_option('prefix'), get_option('datadir'))
localedir  = join_paths(get_option('prefix'), get_option('localedir'))
bindir     = join_paths(get_option('prefix'), get_option('bindir'))

app_id = 'io.github.tsirysndr.Rockbox'
prefix = '/io/github/tsirysndr/Rockbox'

conf = configuration_data()
conf.set('APP_ID', app_id)
conf.set('PREFIX', prefix)
conf.set('VERSION', meson.project_version())
conf.set('pkgdatadir', pkgdatadir)

subdir('src')
subdir('data')
