project('blueprint-compiler',
  version: '0.14.0',
)

prefix = get_option('prefix')
datadir = join_paths(prefix, get_option('datadir'))

py = import('python').find_installation('python3')

subdir('docs')

configure_file(
  input: 'blueprint-compiler.pc.in',
  output: 'blueprint-compiler.pc',
  configuration: { 'VERSION': meson.project_version() },
  install: not meson.is_subproject(),
  install_dir: join_paths(datadir, 'pkgconfig'),
)

config = configuration_data({
  'VERSION': meson.project_version(),
  'LIBDIR': get_option('prefix') / get_option('libdir'),
})

if meson.is_subproject()
  config.set('MODULE_PATH', meson.current_source_dir())
else
  config.set('MODULE_PATH', py.get_install_dir())
endif

blueprint_compiler = configure_file(
  input: 'blueprint-compiler.py',
  output: 'blueprint-compiler',
  configuration: config,
  install: not meson.is_subproject(),
  install_dir: get_option('bindir'),
)

if meson.is_subproject()
  meson.override_find_program('blueprint-compiler', blueprint_compiler)
else
  install_subdir('blueprintcompiler', install_dir: py.get_install_dir())
endif

subdir('tests')
