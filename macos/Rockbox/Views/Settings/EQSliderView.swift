import SwiftUI

// band.cutoff = gain in tenths of dB (-240…+240, i.e. -24.0…+24.0 dB)
// band.q      = frequency identifier; display Hz = q/2  (0 → 16 kHz for last band)
// band.gain   = Q factor — preserved on save, not shown here

struct EQSliderView: View {
    let band: EqBandLocal
    let onChange: (Int) -> Void  // newCutoff in tenths of dB

    private let trackHeight: CGFloat = 160
    private let thumbDiameter: CGFloat = 14

    @State private var isDragging = false

    // fraction 0.0 (= -24 dB) … 1.0 (= +24 dB)
    private func gainToFraction(_ cutoff: Int) -> Double {
        (Double(cutoff) / 10.0 + 24.0) / 48.0
    }

    // slider fraction → stored tenths-of-dB, snapped to 0.1 dB
    private func fractionToGain(_ frac: Double) -> Int {
        let db = frac * 48.0 - 24.0
        return Int((db * 10.0).rounded())
    }

    private var fraction: Double { gainToFraction(band.cutoff) }

    private func freqLabel(q: Int) -> String {
        switch q {
        case 64:    return "32"
        case 125:   return "64"
        case 250:   return "125"
        case 500:   return "250"
        case 1000:  return "500"
        case 2000:  return "1k"
        case 4000:  return "2k"
        case 8000:  return "4k"
        case 16000: return "8k"
        case 0:     return "16k"
        default:    return q >= 1000 ? "\(q / 1000)k" : "\(q)"
        }
    }

    private func formatGain(_ cutoff: Int) -> String {
        let db = Double(cutoff) / 10.0
        if cutoff == 0 { return "0" }
        return String(format: db > 0 ? "+%.1f" : "%.1f", db)
    }

    var body: some View {
        VStack(spacing: 4) {
            Text(formatGain(band.cutoff))
                .font(.system(size: 9, weight: .medium, design: .monospaced))
                .foregroundStyle(.secondary)
                .frame(height: 14)
                .animation(nil, value: band.cutoff)

            GeometryReader { geo in
                let h = geo.size.height
                let centerY = h * 0.5
                let thumbY = h * (1.0 - fraction)
                let fillTop = min(centerY, thumbY)
                let fillHeight = max(0, abs(centerY - thumbY))

                ZStack(alignment: .top) {
                    // Track background
                    RoundedRectangle(cornerRadius: 2)
                        .fill(Color.white.opacity(0.07))
                        .frame(width: 4)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)

                    // Fill from center to thumb
                    if fillHeight > 1 {
                        RoundedRectangle(cornerRadius: 2)
                            .fill(
                                band.cutoff >= 0
                                    ? Color(hex: "fe09a3").opacity(0.85)
                                    : Color(hex: "1a91ff").opacity(0.85)
                            )
                            .frame(width: 4, height: fillHeight)
                            .frame(maxWidth: .infinity, alignment: .center)
                            .offset(y: fillTop)
                            .allowsHitTesting(false)
                    }

                    // Zero-dB center line
                    Rectangle()
                        .fill(Color.white.opacity(0.25))
                        .frame(width: 12, height: 1)
                        .frame(maxWidth: .infinity)
                        .offset(y: centerY - 0.5)
                        .allowsHitTesting(false)

                    // Thumb
                    Circle()
                        .fill(isDragging ? Color.white : Color(white: 0.82))
                        .frame(width: thumbDiameter, height: thumbDiameter)
                        .shadow(color: .black.opacity(0.45), radius: 2, y: 1)
                        .frame(maxWidth: .infinity)
                        .offset(y: thumbY - thumbDiameter / 2)
                        .allowsHitTesting(false)
                }
                .contentShape(Rectangle())
                .gesture(
                    DragGesture(minimumDistance: 0)
                        .onChanged { value in
                            isDragging = true
                            let newFrac = max(0.0, min(1.0, 1.0 - value.location.y / h))
                            onChange(fractionToGain(newFrac))
                        }
                        .onEnded { _ in isDragging = false }
                )
            }
            .frame(width: 24, height: trackHeight)

            Text(freqLabel(q: band.q))
                .font(.system(size: 9, weight: .medium))
                .foregroundStyle(.secondary)
                .frame(height: 14)
        }
        .frame(width: 36)
    }
}
