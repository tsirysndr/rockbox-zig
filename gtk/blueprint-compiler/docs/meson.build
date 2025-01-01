if get_option('docs')

sphinx = find_program(['sphinx-build-3', 'sphinx-build'], required: true)

custom_target('docs',
  command: [sphinx, '-b', 'html', '-c', meson.current_source_dir(), meson.current_source_dir(), '@OUTPUT@'],
  output: 'en',
  build_always_stale: true,
)

endif

custom_target('reference_docs.json',
  output: 'reference_docs.json',
  command: [meson.current_source_dir() / 'collect-sections.py', '@OUTPUT@'],
  build_always_stale: true,
  install: true,
  install_dir: py.get_install_dir() / 'blueprintcompiler',
)