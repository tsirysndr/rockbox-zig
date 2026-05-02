# frozen_string_literal: true

require_relative "lib/rockbox/version"

Gem::Specification.new do |spec|
  spec.name        = "rockbox"
  spec.version     = Rockbox::VERSION
  spec.authors     = ["Tsiry Sandratraina"]
  spec.email       = ["tsiry.sndr@rocksky.app"]

  spec.summary     = "Idiomatic Ruby SDK for Rockbox"
  spec.description = "Ruby SDK for Rockbox — a builder-friendly, block-friendly GraphQL client " \
                     "with real-time event subscriptions and a plugin system."
  spec.homepage    = "https://github.com/tsirysndr/rockbox-zig"
  spec.license     = "MIT"

  spec.required_ruby_version = ">= 3.0"

  spec.metadata["homepage_uri"]      = spec.homepage
  spec.metadata["source_code_uri"]   = "#{spec.homepage}/tree/master/sdk/ruby"
  spec.metadata["bug_tracker_uri"]   = "#{spec.homepage}/issues"
  spec.metadata["documentation_uri"] = "https://www.rockbox.org"

  spec.files = Dir[
    "lib/**/*.rb",
    "README.md",
    "rockbox.gemspec"
  ]
  spec.require_paths = ["lib"]

  spec.add_dependency "websocket-client-simple", "~> 0.6"

  spec.add_development_dependency "bundler", "~> 4.0"
  spec.add_development_dependency "minitest", "~> 5.0"
  spec.add_development_dependency "rake", "~> 13.0"
end
