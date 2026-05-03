import { useState } from "react";
import {
  GestureResponderEvent,
  LayoutChangeEvent,
  PanResponder,
  View,
  ViewStyle,
} from "react-native";
import { Colors } from "@/constants/theme";

export function SeekBar({
  value,
  max,
  onSeek,
  height = 4,
  thumb = true,
  track = Colors.sliderTrack,
  fill = Colors.sliderFill,
  style,
}: {
  value: number;
  max: number;
  onSeek?: (newValue: number) => void;
  height?: number;
  thumb?: boolean;
  track?: string;
  fill?: string;
  style?: ViewStyle;
}) {
  const [width, setWidth] = useState(0);
  const [drag, setDrag] = useState<number | null>(null);

  const fraction =
    drag !== null
      ? Math.min(1, Math.max(0, drag))
      : max > 0
        ? Math.min(1, Math.max(0, value / max))
        : 0;

  const compute = (event: GestureResponderEvent): number => {
    const x = event.nativeEvent.locationX;
    if (width <= 0) return 0;
    return Math.min(1, Math.max(0, x / width));
  };

  const responder = PanResponder.create({
    onStartShouldSetPanResponder: () => true,
    onMoveShouldSetPanResponder: () => true,
    onPanResponderGrant: (e) => setDrag(compute(e)),
    onPanResponderMove: (e) => setDrag(compute(e)),
    onPanResponderRelease: (e) => {
      const final = compute(e);
      setDrag(null);
      onSeek?.(final * max);
    },
    onPanResponderTerminate: () => setDrag(null),
  });

  return (
    <View
      onLayout={(e: LayoutChangeEvent) => setWidth(e.nativeEvent.layout.width)}
      {...responder.panHandlers}
      className="h-6 justify-center"
      style={style}
    >
      <View
        className="overflow-hidden"
        style={{
          height,
          backgroundColor: track,
          borderRadius: height / 2,
        }}
      >
        <View
          className="h-full"
          style={{
            width: `${fraction * 100}%`,
            backgroundColor: fill,
          }}
        />
      </View>
      {thumb ? (
        <View
          pointerEvents="none"
          className="absolute w-3 h-3 rounded-full bg-white -ml-1.5"
          style={{
            left: `${fraction * 100}%`,
            shadowColor: "#000",
            shadowOpacity: 0.5,
            shadowRadius: 4,
            shadowOffset: { width: 0, height: 1 },
          }}
        />
      ) : null}
    </View>
  );
}
