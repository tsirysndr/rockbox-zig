# frozen_string_literal: true

module Rockbox
  # Convert between GraphQL camelCase JSON keys and Ruby snake_case symbols.
  # The transport applies these on every request/response so user code can
  # always work in idiomatic Ruby.
  module CaseConversion
    module_function

    def camelize(str)
      head, *tail = str.to_s.split("_")
      ([head] + tail.map(&:capitalize)).join
    end

    def snakeize(str)
      str.to_s.gsub(/([a-z\d])([A-Z])/, '\1_\2').downcase
    end

    # Recursively rewrite a Hash/Array — used on outgoing variable payloads.
    def deep_camelize(obj)
      case obj
      when Hash
        obj.each_with_object({}) { |(k, v), h| h[camelize(k)] = deep_camelize(v) }
      when Array
        obj.map { |v| deep_camelize(v) }
      else
        obj
      end
    end

    # Recursively rewrite a Hash/Array — used on incoming GraphQL responses.
    def deep_snakeize(obj)
      case obj
      when Hash
        obj.each_with_object({}) { |(k, v), h| h[snakeize(k).to_sym] = deep_snakeize(v) }
      when Array
        obj.map { |v| deep_snakeize(v) }
      else
        obj
      end
    end
  end
end
