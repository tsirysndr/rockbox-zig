# frozen_string_literal: true

require_relative "test_helper"

class ErrorsTest < Minitest::Test
  def test_error_carries_cause
    cause = StandardError.new("inner")
    err = Rockbox::Error.new("outer", cause: cause)
    assert_equal "outer", err.message
    assert_same cause, err.cause
  end

  def test_network_error_is_an_error
    assert_kind_of Rockbox::Error, Rockbox::NetworkError.new("oops")
  end

  def test_graphql_error_concatenates_messages_with_symbol_keys
    err = Rockbox::GraphQLError.new([
      { message: "first" },
      { message: "second" }
    ])
    assert_equal "first; second", err.message
    assert_equal 2, err.errors.size
  end

  def test_graphql_error_concatenates_messages_with_string_keys
    err = Rockbox::GraphQLError.new([
      { "message" => "boom" }
    ])
    assert_equal "boom", err.message
  end

  def test_graphql_error_handles_messageless_entries
    err = Rockbox::GraphQLError.new([{ message: "ok" }, { code: "X" }])
    assert_equal "ok", err.message
  end
end
