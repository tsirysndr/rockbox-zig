project('rockbox-gtk', 'rust',
          version: '0.1.0',
    meson_version: '>= 0.59.0',
          license: 'GPL-3.0-or-later',
)

gnome = import('gnome')
i18n = import('i18n')

# Dependencies

dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.0.0')
dependency('libadwaita-1', version: '>=1.6.beta')

# Initialize project variables
name       = 'Rockbox'
base_id    = 'mg.tsirysndr.Rockbox'
path_id    = '/mg/tsirysndr/Rockbox'
app_id     = base_id
vcs_tag    = run_command('git', 'rev-parse', '--short', 'HEAD', check: false).stdout().strip()
datadir    = join_paths(get_option('prefix'), get_option('datadir'))
localedir  = join_paths(get_option('prefix'), get_option('localedir'))
bindir     = join_paths(get_option('prefix'), get_option('bindir'))

subdir('src')