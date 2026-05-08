import Foundation
import SwiftUI

struct EqBandLocal: Identifiable {
    let id: Int
    var cutoff: Int  // user gain in tenths of dB (-240…+240), i.e. -24.0…+24.0 dB
    var q: Int       // display identifier only — maps to freqLabel() in EQSliderView
    var gain: Int    // unused in save; kept for local state only
}

// Canonical center frequencies (Hz) sent to the firmware as wire cutoff.
private let bandFreqs: [Int32] = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000]
// Valid Q factors (tenths, range 1-64): shelving bands 0/9 use Q=0.7, peaks use Q=1.0.
private let bandQ: [Int32] = [7, 10, 10, 10, 10, 10, 10, 10, 10, 7]
// Display-only identifiers used by EQSliderView.freqLabel() — never sent to firmware.
private let displayQValues = [64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000, 0]
// Canonical frequencies as a Set for O(1) migration checks.
private let canonicalFreqSet: Set<Int> = [32, 64, 125, 250, 500, 1000, 2000, 4000, 8000, 16000]

func defaultEqBands() -> [EqBandLocal] {
    displayQValues.enumerated().map { EqBandLocal(id: $0.offset, cutoff: 0, q: $0.element, gain: 0) }
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

    // Build the wire EqBandSetting array from local slider state.
    // Matches the web UI convention exactly so all clients are compatible:
    //   wire.cutoff = user gain (tenths dB)   ← what the web UI reads/writes
    //   wire.q      = pass-through identifier ← web UI uses this for band labels
    //   wire.gain   = 0                       ← keeps DSP filters disabled → no noise
    private func wireEqBands() -> [Rockbox_V1alpha1_EqBandSetting] {
        eqBands.map { band in
            var s = Rockbox_V1alpha1_EqBandSetting()
            s.cutoff = Int32(band.cutoff)   // user gain (tenths dB)
            s.q      = Int32(band.q)        // display identifier (pass-through)
            s.gain   = 0                    // always 0 — DSP disabled, no noise
            return s
        }
    }

    func scheduleEqBandsApply() {
        eqDebounceTask?.cancel()
        eqDebounceTask = Task {
            do { try await Task.sleep(for: .milliseconds(80)) } catch { return }
            do {
                var req = Rockbox_V1alpha1_SaveSettingsRequest()
                req.eqBandSettings = wireEqBands()
                try await saveAllSettings(req)
            } catch {}
        }
    }

    func applyEqEnabled() {
        Task {
            do {
                var req = Rockbox_V1alpha1_SaveSettingsRequest()
                req.eqEnabled = eqEnabled
                // Always include current band settings so write_settings() on the server
                // flushes valid parameters to disk (not whatever stale values are in memory).
                req.eqBandSettings = wireEqBands()
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
                    // Normalize server state to local convention: EqBandLocal.cutoff = user gain.
                    // Three cases:
                    //   A) old GPUI bug: gain in wire.gain (non-zero), cutoff = frequency Hz
                    //   B) firmware default: cutoff = frequency Hz, gain = 0
                    //   C) web UI / correct: cutoff = user gain, gain = 0
                    eqBands = data.eqBandSettings.enumerated().map { i, band in
                        let userGain: Int
                        if band.gain != 0 {
                            userGain = Int(band.gain)                    // case A
                        } else if canonicalFreqSet.contains(Int(band.cutoff)) {
                            userGain = 0                                 // case B
                        } else {
                            userGain = Int(band.cutoff)                  // case C
                        }
                        let dq = i < displayQValues.count ? displayQValues[i] : 0
                        return EqBandLocal(id: i, cutoff: userGain, q: dq, gain: 0)
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
                req.eqBandSettings = wireEqBands()
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
