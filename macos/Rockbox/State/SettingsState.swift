import Foundation
import SwiftUI

struct EqBandLocal: Identifiable {
    let id: Int
    var cutoff: Int  // gain in tenths of dB (-240…+240), i.e. -24.0…+24.0 dB
    var q: Int       // frequency identifier; display Hz = q/2  (0 = 16 kHz for last band)
    var gain: Int    // Q factor — preserved on save, not edited here
}

// q values mirror the server encoding: 64 → 32 Hz band, 125 → 64 Hz, …, 0 → 16 kHz
private let defaultQValues = [64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000, 0]

func defaultEqBands() -> [EqBandLocal] {
    defaultQValues.enumerated().map { EqBandLocal(id: $0.offset, cutoff: 0, q: $0.element, gain: 10) }
}

enum SettingsTab: String, CaseIterable {
    case general = "General"
    case equalizer = "Equalizer"
    case playback = "Playback"
    case sound = "Sound"
}

@MainActor
class SettingsState: ObservableObject {
    @Published var activeTab: SettingsTab = .general

    // General
    @Published var musicDir = ""
    @Published var playerName = ""

    // Equalizer
    @Published var eqEnabled = false
    @Published var eqBands: [EqBandLocal] = defaultEqBands()

    // Playback
    @Published var shuffle = false
    @Published var crossfade = 0
    @Published var fadeInDelay = 0
    @Published var fadeInDuration = 0
    @Published var fadeOutDelay = 0
    @Published var fadeOutDuration = 0
    @Published var fadeOutMixmode = 0
    @Published var replaygainType = 0
    @Published var replaygainPreamp = 0
    @Published var replaygainNoclip = false

    // Sound
    @Published var balance = 0
    @Published var bass = 0
    @Published var treble = 0
    @Published var stereoWidth = 100
    @Published var channelConfig = 0
    @Published var surroundEnabled = 0

    @Published var isLoading = false
    @Published var isSaving = false
    @Published var errorMessage: String? = nil

    private var eqDebounceTask: Task<Void, Never>?

    func scheduleEqBandsApply() {
        eqDebounceTask?.cancel()
        eqDebounceTask = Task {
            do { try await Task.sleep(for: .milliseconds(80)) } catch { return }
            do {
                var req = Rockbox_V1alpha1_SaveSettingsRequest()
                req.eqBandSettings = eqBands.map { band in
                    var s = Rockbox_V1alpha1_EqBandSetting()
                    s.cutoff = Int32(band.cutoff)
                    s.q = Int32(band.q)
                    s.gain = Int32(band.gain)
                    return s
                }
                try await saveAllSettings(req)
            } catch {}
        }
    }

    func applyEqEnabled() {
        Task {
            do {
                var req = Rockbox_V1alpha1_SaveSettingsRequest()
                req.eqEnabled = eqEnabled
                try await saveAllSettings(req)
            } catch {}
        }
    }

    func load() {
        isLoading = true
        errorMessage = nil
        Task {
            do {
                let data = try await fetchGlobalSettings()
                musicDir = data.musicDir
                playerName = data.playerName
                eqEnabled = data.eqEnabled
                if !data.eqBandSettings.isEmpty {
                    eqBands = data.eqBandSettings.enumerated().map { i, band in
                        EqBandLocal(id: i, cutoff: Int(band.cutoff), q: Int(band.q), gain: Int(band.gain))
                    }
                }
                shuffle = data.playlistShuffle
                crossfade = Int(data.crossfade)
                fadeInDelay = Int(data.crossfadeFadeInDelay)
                fadeInDuration = Int(data.crossfadeFadeInDuration)
                fadeOutDelay = Int(data.crossfadeFadeOutDelay)
                fadeOutDuration = Int(data.crossfadeFadeOutDuration)
                fadeOutMixmode = Int(data.crossfadeFadeOutMixmode)
                replaygainType = Int(data.replaygainSettings.type)
                replaygainPreamp = Int(data.replaygainSettings.preamp)
                replaygainNoclip = data.replaygainSettings.noclip
                balance = Int(data.balance)
                bass = Int(data.bass)
                treble = Int(data.treble)
                stereoWidth = Int(data.stereoWidth)
                channelConfig = Int(data.channelConfig)
                surroundEnabled = Int(data.surroundEnabled)
            } catch {
                errorMessage = error.localizedDescription
            }
            isLoading = false
        }
    }

    func save() {
        isSaving = true
        errorMessage = nil
        Task {
            do {
                var req = Rockbox_V1alpha1_SaveSettingsRequest()
                req.musicDir = musicDir
                req.playerName = playerName
                req.eqEnabled = eqEnabled
                req.eqBandSettings = eqBands.map { band in
                    var s = Rockbox_V1alpha1_EqBandSetting()
                    s.cutoff = Int32(band.cutoff)
                    s.q = Int32(band.q)
                    s.gain = Int32(band.gain)
                    return s
                }
                req.playlistShuffle = shuffle
                req.crossfade = Int32(crossfade)
                req.fadeInDelay = Int32(fadeInDelay)
                req.fadeInDuration = Int32(fadeInDuration)
                req.fadeOutDelay = Int32(fadeOutDelay)
                req.fadeOutDuration = Int32(fadeOutDuration)
                req.fadeOutMixmode = Int32(fadeOutMixmode)
                var rg = Rockbox_V1alpha1_ReplaygainSettings()
                rg.type = Int32(replaygainType)
                rg.preamp = Int32(replaygainPreamp)
                rg.noclip = replaygainNoclip
                req.replaygainSettings = rg
                req.balance = Int32(balance)
                req.bass = Int32(bass)
                req.treble = Int32(treble)
                req.stereoWidth = Int32(stereoWidth)
                req.channelConfig = Int32(channelConfig)
                req.surroundEnabled = Int32(surroundEnabled)
                try await saveAllSettings(req)
            } catch {
                errorMessage = error.localizedDescription
            }
            isSaving = false
        }
    }
}
