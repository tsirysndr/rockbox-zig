# frozen_string_literal: true

require_relative "../types"

module Rockbox
  module Api
    class Settings
      QUERY = <<~GQL
        query GlobalSettings {
          globalSettings {
            musicDir volume balance bass treble channelConfig stereoWidth
            eqEnabled eqPrecut
            eqBandSettings { cutoff q gain }
            replaygainSettings { noclip type preamp }
            compressorSettings { threshold makeupGain ratio knee releaseTime attackTime }
            crossfadeEnabled crossfadeFadeInDelay crossfadeFadeInDuration
            crossfadeFadeOutDelay crossfadeFadeOutDuration crossfadeFadeOutMixmode
            crossfeedEnabled crossfeedDirectGain crossfeedCrossGain
            crossfeedHfAttenuation crossfeedHfCutoff
            repeatMode singleMode partyMode shuffle playerName
          }
        }
      GQL

      def initialize(http)
        @http = http
      end

      # @return [Rockbox::UserSettings]
      def get
        s = @http.execute(QUERY)[:global_settings] || {}

        UserSettings.new(
          music_dir:                   s[:music_dir],
          volume:                      s[:volume],
          balance:                     s[:balance],
          bass:                        s[:bass],
          treble:                      s[:treble],
          channel_config:              s[:channel_config],
          stereo_width:                s[:stereo_width],
          eq_enabled:                  s[:eq_enabled],
          eq_precut:                   s[:eq_precut],
          eq_band_settings:            Array(s[:eq_band_settings]).map { |b| EqBandSetting.from_hash(b) },
          replaygain_settings:         ReplaygainSettings.from_hash(s[:replaygain_settings]),
          compressor_settings:         CompressorSettings.from_hash(s[:compressor_settings]),
          crossfade_enabled:           s[:crossfade_enabled],
          crossfade_fade_in_delay:     s[:crossfade_fade_in_delay],
          crossfade_fade_in_duration:  s[:crossfade_fade_in_duration],
          crossfade_fade_out_delay:    s[:crossfade_fade_out_delay],
          crossfade_fade_out_duration: s[:crossfade_fade_out_duration],
          crossfade_fade_out_mixmode:  s[:crossfade_fade_out_mixmode],
          crossfeed_enabled:           s[:crossfeed_enabled],
          crossfeed_direct_gain:       s[:crossfeed_direct_gain],
          crossfeed_cross_gain:        s[:crossfeed_cross_gain],
          crossfeed_hf_attenuation:    s[:crossfeed_hf_attenuation],
          crossfeed_hf_cutoff:         s[:crossfeed_hf_cutoff],
          repeat_mode:                 s[:repeat_mode],
          single_mode:                 s[:single_mode],
          party_mode:                  s[:party_mode],
          shuffle:                     s[:shuffle],
          player_name:                 s[:player_name]
        )
      end

      # Save a partial settings update. Pass any subset of keys; everything
      # else is left as-is by the firmware.
      #
      # @example Builder block
      #   client.settings.save do |s|
      #     s.volume = -20
      #     s.bass = 4
      #     s.shuffle = true
      #   end
      #
      # @example Hash
      #   client.settings.save(volume: -20, bass: 4)
      def save(settings = nil, &block)
        if block
          builder = SaveBuilder.new
          yield builder
          settings = builder.to_h.merge(settings || {})
        end
        raise ArgumentError, "settings hash or block required" if settings.nil? || settings.empty?

        @http.execute(
          "mutation SaveSettings($settings: NewGlobalSettings!) { saveSettings(settings: $settings) }",
          { settings: settings }
        )
        nil
      end

      class SaveBuilder
        SETTABLE = %i[
          music_dir volume balance bass treble channel_config stereo_width
          eq_enabled eq_precut eq_band_settings replaygain_settings compressor_settings
          crossfade_enabled crossfade_fade_in_delay crossfade_fade_in_duration
          crossfade_fade_out_delay crossfade_fade_out_duration crossfade_fade_out_mixmode
          crossfeed_enabled crossfeed_direct_gain crossfeed_cross_gain
          crossfeed_hf_attenuation crossfeed_hf_cutoff
          repeat_mode single_mode party_mode shuffle player_name
        ].freeze

        def initialize
          @attrs = {}
        end

        SETTABLE.each do |attr|
          define_method("#{attr}=") { |value| @attrs[attr] = value }
          define_method(attr)        { @attrs[attr] }
        end

        def to_h
          @attrs.dup
        end
      end
    end
  end
end
