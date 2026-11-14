project('pole-rust-example-plugin',
  version: '1.0.0',
  meson_version: '>= 0.59.0',
  license: 'MIT',
)

plugins_install_dir = get_option('datadir') / 'neomir' / 'plugins' / 'rust-example'

plugin_sources = files(
  'src/main.rs',
)

cargo_script = meson.current_source_dir() / 'build-cargo.sh'

rust_plugin_bin = custom_target(
  'rust-example-plugin',
  build_by_default: true,
  build_always_stale: true,
  input: plugin_sources,
  output: 'rust-example-plugin',
  console: true,
  install: true,
  install_dir: plugins_install_dir,
  command: [
    cargo_script,
    meson.current_source_dir(),
    meson.current_build_dir(),
    '@OUTPUT@',
    get_option('buildtype'),
    'rust-example-plugin',
  ],
)

install_data(
  'plugin.json',
  install_dir: plugins_install_dir,
)

# Optional .plug package bundle generator for distributing via single file
tar_prog = find_program('tar', required: false)
if tar_prog.found()
  custom_target(
    'rust-example-plug-package',
    input: [
      rust_plugin_bin,
      files('plugin.json'),
    ],
    output: 'rust-example.plug',
    build_by_default: true,
    command: [
      tar_prog,
      '-czf',
      '@OUTPUT@',
      '-C', meson.current_build_dir(),
      'rust-example-plugin',
      '-C', meson.current_source_dir(),
      'plugin.json',
    ],
  )
endif
