import SwiftUI

struct SettingsView: View {
    @StateObject private var state = SettingsState()
    @Environment(\.dismiss) private var dismiss

    var body: some View {
        VStack(spacing: 0) {
            headerBar
            Divider()
            tabPicker
            Divider()
            tabContent
        }
        .frame(width: 560, height: 520)
        .background(.bar)
        .onAppear { state.load() }
        .overlay {
            if state.isLoading {
                ProgressView()
                    .frame(maxWidth: .infinity, maxHeight: .infinity)
                    .background(Color.black.opacity(0.2))
            }
        }
    }

    private var headerBar: some View {
        HStack(spacing: 12) {
            Text("Settings")
                .font(.system(size: 15, weight: .semibold))
            Spacer()
            if let err = state.errorMessage {
                Text(err)
                    .font(.system(size: 11))
                    .foregroundStyle(.red)
                    .lineLimit(1)
                    .truncationMode(.tail)
            }
            Button("Cancel") { dismiss() }
                .buttonStyle(.plain)
                .foregroundStyle(.secondary)
            Button(state.isSaving ? "Saving…" : "Save") {
                state.save()
                dismiss()
            }
            .buttonStyle(.plain)
            .foregroundStyle(Color(hex: "fe09a3"))
            .disabled(state.isSaving)
        }
        .padding(.horizontal, 20)
        .padding(.vertical, 14)
    }

    private var tabPicker: some View {
        Picker("Tab", selection: $state.activeTab) {
            ForEach(SettingsTab.allCases, id: \.self) { tab in
                Text(tab.rawValue).tag(tab)
            }
        }
        .pickerStyle(.segmented)
        .padding(.horizontal, 20)
        .padding(.vertical, 10)
    }

    @ViewBuilder
    private var tabContent: some View {
        ScrollView {
            switch state.activeTab {
            case .general:   GeneralTabView(state: state)
            case .equalizer: EqualizerTabView(state: state)
            case .playback:  PlaybackTabView(state: state)
            case .sound:     SoundTabView(state: state)
            }
        }
        .frame(maxHeight: .infinity)
    }
}

// MARK: - General

private struct GeneralTabView: View {
    @ObservedObject var state: SettingsState

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            SettingsSection("Library") {
                SettingsRow("Music folder") {
                    TextField("Path to music library", text: $state.musicDir)
                        .textFieldStyle(.roundedBorder)
                        .frame(maxWidth: 300)
                }
            }
            SettingsSection("Device") {
                SettingsRow("Player name") {
                    TextField("e.g. Living Room", text: $state.playerName)
                        .textFieldStyle(.roundedBorder)
                        .frame(maxWidth: 200)
                }
            }
        }
        .padding(.bottom, 20)
    }
}

// MARK: - Equalizer

private struct EqualizerTabView: View {
    @ObservedObject var state: SettingsState

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            SettingsSection("Equalizer") {
                SettingsRow("Enable EQ") {
                    Toggle("", isOn: $state.eqEnabled)
                        .toggleStyle(.switch)
                        .labelsHidden()
                        .onChange(of: state.eqEnabled) { _ in state.applyEqEnabled() }
                }
            }

            VStack(spacing: 6) {
                HStack(alignment: .bottom, spacing: 2) {
                    ForEach(state.eqBands) { band in
                        EQSliderView(band: band) { cutoff in
                            var updated = state.eqBands
                            updated[band.id].cutoff = cutoff
                            state.eqBands = updated
                            state.scheduleEqBandsApply()
                        }
                    }
                }
                .frame(maxWidth: .infinity)
                .padding(.horizontal, 20)
            }
            .padding(.top, 12)
            .padding(.bottom, 20)
            .opacity(state.eqEnabled ? 1.0 : 0.35)
            .animation(.easeInOut(duration: 0.2), value: state.eqEnabled)
        }
        .padding(.bottom, 20)
    }
}

// MARK: - Playback

private struct PlaybackTabView: View {
    @ObservedObject var state: SettingsState

    private let crossfadeModes = [
        "Off", "Auto Track Change", "Manual Track Skip",
        "Shuffle", "Shuffle and Track Skip"
    ]
    private let mixmodes = ["Crossfade", "Mix"]
    private let replaygainTypes = ["Track Gain", "Album Gain", "Track Gain if Shuffle", "Off"]

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            SettingsSection("Playback") {
                SettingsRow("Shuffle") {
                    Toggle("", isOn: $state.shuffle)
                        .toggleStyle(.switch)
                        .labelsHidden()
                }
            }

            SettingsSection("Crossfade") {
                SettingsRow("Mode") {
                    Picker("", selection: $state.crossfade) {
                        ForEach(crossfadeModes.indices, id: \.self) { i in
                            Text(crossfadeModes[i]).tag(i)
                        }
                    }
                    .pickerStyle(.menu)
                    .labelsHidden()
                    .frame(width: 210)
                }
                SettingsRow("Fade in delay") {
                    LabeledStepper(value: $state.fadeInDelay, in: 0...7, unit: "s")
                }
                SettingsRow("Fade in duration") {
                    LabeledStepper(value: $state.fadeInDuration, in: 0...15, unit: "s")
                }
                SettingsRow("Fade out delay") {
                    LabeledStepper(value: $state.fadeOutDelay, in: 0...7, unit: "s")
                }
                SettingsRow("Fade out duration") {
                    LabeledStepper(value: $state.fadeOutDuration, in: 0...15, unit: "s")
                }
                SettingsRow("Fade out mix mode") {
                    Picker("", selection: $state.fadeOutMixmode) {
                        ForEach(mixmodes.indices, id: \.self) { i in
                            Text(mixmodes[i]).tag(i)
                        }
                    }
                    .pickerStyle(.menu)
                    .labelsHidden()
                    .frame(width: 140)
                }
            }

            SettingsSection("ReplayGain") {
                SettingsRow("Type") {
                    Picker("", selection: $state.replaygainType) {
                        ForEach(replaygainTypes.indices, id: \.self) { i in
                            Text(replaygainTypes[i]).tag(i)
                        }
                    }
                    .pickerStyle(.menu)
                    .labelsHidden()
                    .frame(width: 210)
                }
                SettingsRow("Preamp") {
                    LabeledStepper(value: $state.replaygainPreamp, in: -120...120, unit: "dB/10")
                }
                SettingsRow("Prevent clipping") {
                    Toggle("", isOn: $state.replaygainNoclip)
                        .toggleStyle(.switch)
                        .labelsHidden()
                }
            }
        }
        .padding(.bottom, 20)
    }
}

// MARK: - Sound

private struct SoundTabView: View {
    @ObservedObject var state: SettingsState

    private let channelConfigs = [
        "Stereo", "Mono", "Custom", "Mono Left",
        "Mono Right", "Stereo Narrow", "Karaoke"
    ]

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            SettingsSection("Sound") {
                SettingsRow("Balance") {
                    SettingsSlider(
                        value: $state.balance,
                        in: -100...100,
                        label: { v in v == 0 ? "Center" : (v > 0 ? "R \(v)" : "L \(-v)") }
                    )
                }
                SettingsRow("Bass") {
                    SettingsSlider(
                        value: $state.bass,
                        in: -240...240,
                        label: { v in String(format: "%+.1f dB", Double(v) / 10.0) }
                    )
                }
                SettingsRow("Treble") {
                    SettingsSlider(
                        value: $state.treble,
                        in: -240...240,
                        label: { v in String(format: "%+.1f dB", Double(v) / 10.0) }
                    )
                }
                SettingsRow("Stereo width") {
                    SettingsSlider(
                        value: $state.stereoWidth,
                        in: 0...255,
                        label: { v in "\(v)%" }
                    )
                }
                SettingsRow("Channel config") {
                    Picker("", selection: $state.channelConfig) {
                        ForEach(channelConfigs.indices, id: \.self) { i in
                            Text(channelConfigs[i]).tag(i)
                        }
                    }
                    .pickerStyle(.menu)
                    .labelsHidden()
                    .frame(width: 160)
                }
                SettingsRow("Surround") {
                    LabeledStepper(value: $state.surroundEnabled, in: 0...7, unit: "")
                }
            }
        }
        .padding(.bottom, 20)
    }
}

// MARK: - Shared helpers

private struct SettingsSection<Content: View>: View {
    let title: String
    @ViewBuilder let content: () -> Content

    init(_ title: String, @ViewBuilder content: @escaping () -> Content) {
        self.title = title
        self.content = content
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 0) {
            Text(title)
                .font(.system(size: 11, weight: .semibold))
                .foregroundStyle(.secondary)
                .textCase(.uppercase)
                .padding(.horizontal, 20)
                .padding(.top, 18)
                .padding(.bottom, 8)

            VStack(spacing: 0) {
                content()
            }
            .background(Color.white.opacity(0.04))
            .cornerRadius(8)
            .padding(.horizontal, 16)
        }
    }
}

private struct SettingsRow<Control: View>: View {
    let label: String
    @ViewBuilder let control: () -> Control

    init(_ label: String, @ViewBuilder control: @escaping () -> Control) {
        self.label = label
        self.control = control
    }

    var body: some View {
        HStack {
            Text(label)
                .font(.system(size: 13))
            Spacer()
            control()
        }
        .padding(.horizontal, 14)
        .padding(.vertical, 9)
        .overlay(alignment: .bottom) {
            Divider().padding(.leading, 14)
        }
    }
}

private struct LabeledStepper: View {
    @Binding var value: Int
    let range: ClosedRange<Int>
    let unit: String

    init(value: Binding<Int>, in range: ClosedRange<Int>, unit: String) {
        _value = value
        self.range = range
        self.unit = unit
    }

    var body: some View {
        HStack(spacing: 6) {
            Text(unit.isEmpty ? "\(value)" : "\(value) \(unit)")
                .font(.system(size: 12, design: .monospaced))
                .foregroundStyle(.secondary)
                .frame(width: unit.isEmpty ? 24 : 70, alignment: .trailing)
            Stepper("", value: $value, in: range)
                .labelsHidden()
        }
    }
}

private struct SettingsSlider: View {
    @Binding var value: Int
    let range: ClosedRange<Int>
    let label: (Int) -> String

    init(value: Binding<Int>, in range: ClosedRange<Int>, label: @escaping (Int) -> String) {
        _value = value
        self.range = range
        self.label = label
    }

    var body: some View {
        HStack(spacing: 8) {
            Slider(
                value: Binding(
                    get: { Double(value) },
                    set: { value = Int($0.rounded()) }
                ),
                in: Double(range.lowerBound)...Double(range.upperBound)
            )
            .frame(width: 160)
            Text(label(value))
                .font(.system(size: 11, design: .monospaced))
                .foregroundStyle(.secondary)
                .frame(width: 76, alignment: .leading)
        }
    }
}
