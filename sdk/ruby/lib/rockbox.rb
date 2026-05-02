# frozen_string_literal: true

require_relative "rockbox/version"
require_relative "rockbox/errors"
require_relative "rockbox/types"
require_relative "rockbox/case_conversion"
require_relative "rockbox/configuration"
require_relative "rockbox/transport"
require_relative "rockbox/events"
require_relative "rockbox/plugin"
require_relative "rockbox/client"

# Top-level convenience constructor — `Rockbox.new(host: "...")` is a synonym
# for `Rockbox::Client.new(host: "...")`.
module Rockbox
  def self.new(**kwargs, &block)
    block ? Client.build(&block) : Client.new(**kwargs)
  end
end
