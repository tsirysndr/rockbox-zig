require 'json'

package = JSON.parse(File.read(File.join(__dir__, '..', 'package.json')))

Pod::Spec.new do |s|
  s.name           = 'RockboxRpc'
  s.version        = package['version']
  s.summary        = package['description']
  s.description    = package['description']
  s.license        = 'LGPL-2.1'
  s.author         = 'Rockbox Daemon'
  s.homepage       = 'https://github.com/tsirysndr/rockboxd'
  s.platforms      = { :ios => '15.1', :tvos => '15.1' }
  s.swift_version  = '5.9'
  s.source         = { git: '' }
  s.static_framework = true

  s.dependency 'ExpoModulesCore'

  # Build the Rust static library before linking. The xcframework is produced
  # by `scripts/build-ios.sh` and dropped into `ios/RockboxExpo.xcframework`.
  s.vendored_frameworks = 'RockboxExpo.xcframework'
  s.libraries = 'c++', 'resolv'

  s.pod_target_xcconfig = {
    'DEFINES_MODULE'             => 'YES',
    'SWIFT_COMPILATION_MODE'     => 'wholemodule',
  }

  s.source_files = 'RockboxRpcModule.swift'
end
