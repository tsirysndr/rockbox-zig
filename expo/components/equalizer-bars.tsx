import { useEffect, useRef } from "react";
import { Animated, Easing, View } from "react-native";

import { Colors } from "@/constants/theme";

/**
 * Spotify/Tidal-style "now playing" indicator: three thin bars that bounce
 * up and down at slightly offset cadences while `playing` is true. Pauses
 * (collapses to 1/4 height) when `playing` is false.
 *
 * Cheap — three reanimated values, no per-frame JS work.
 */
export function EqualizerBars({
  size = 14,
  color = Colors.accent,
  playing = true,
}: {
  size?: number;
  color?: string;
  playing?: boolean;
}) {
  const bars = useRef([
    new Animated.Value(0.4),
    new Animated.Value(0.7),
    new Animated.Value(0.5),
  ]).current;

  useEffect(() => {
    if (!playing) {
      bars.forEach((v) => v.setValue(0.25));
      return;
    }
    const animations = bars.map((v, i) => {
      const phase = [
        { dur: 380, min: 0.2, max: 1 },
        { dur: 510, min: 0.35, max: 0.9 },
        { dur: 440, min: 0.25, max: 0.95 },
      ][i];
      const seq = () =>
        Animated.sequence([
          Animated.timing(v, {
            toValue: phase.max,
            duration: phase.dur,
            easing: Easing.inOut(Easing.quad),
            useNativeDriver: false,
          }),
          Animated.timing(v, {
            toValue: phase.min,
            duration: phase.dur,
            easing: Easing.inOut(Easing.quad),
            useNativeDriver: false,
          }),
        ]);
      const loop = Animated.loop(seq());
      loop.start();
      return loop;
    });
    return () => animations.forEach((a) => a.stop());
  }, [playing, bars]);

  const barWidth = Math.max(2, Math.round(size / 6));
  const gap = Math.max(1, Math.round(size / 10));

  return (
    <View
      style={{
        width: barWidth * 3 + gap * 2,
        height: size,
        flexDirection: "row",
        alignItems: "flex-end",
        justifyContent: "space-between",
      }}
    >
      {bars.map((v, i) => (
        <Animated.View
          key={i}
          style={{
            width: barWidth,
            backgroundColor: color,
            borderRadius: barWidth / 2,
            height: v.interpolate({
              inputRange: [0, 1],
              outputRange: [size * 0.15, size],
            }),
          }}
        />
      ))}
    </View>
  );
}
