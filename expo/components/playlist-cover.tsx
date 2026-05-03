import { Ionicons } from "@expo/vector-icons";
import { Image } from "expo-image";
import { LinearGradient } from "expo-linear-gradient";
import { View } from "react-native";

const GRADIENTS: [string, string][] = [
  ["#FF6B6B", "#C44569"],
  ["#FFA751", "#FFE259"],
  ["#43E97B", "#38F9D7"],
  ["#4FACFE", "#00F2FE"],
  ["#A18CD1", "#FBC2EB"],
  ["#5EE7DF", "#B490CA"],
  ["#FBAB7E", "#F7CE68"],
  ["#FF9A8B", "#FF6A88"],
  ["#667EEA", "#764BA2"],
  ["#F093FB", "#F5576C"],
  ["#30CFD0", "#330867"],
  ["#FA709A", "#FEE140"],
  ["#84FAB0", "#8FD3F4"],
  ["#A8EDEA", "#FED6E3"],
];

function hashSeed(seed?: string): number {
  if (!seed) return 0;
  let h = 0;
  for (let i = 0; i < seed.length; i++) {
    h = (h * 31 + seed.charCodeAt(i)) | 0;
  }
  return Math.abs(h);
}

export function gradientColors(seed?: string): [string, string] {
  return GRADIENTS[hashSeed(seed) % GRADIENTS.length];
}

export function PlaylistCover({
  artwork,
  seed,
  size,
  rounded = "md",
  iconSize,
  className,
}: {
  artwork?: string | null;
  seed?: string;
  size: number;
  rounded?: "sm" | "md" | "lg" | "xl";
  iconSize?: number;
  className?: string;
}) {
  const radiusClass =
    rounded === "sm"
      ? "rounded"
      : rounded === "lg"
        ? "rounded-lg"
        : rounded === "xl"
          ? "rounded-2xl"
          : "rounded-md";
  if (artwork) {
    return (
      <Image
        source={artwork}
        style={{ width: size, height: size }}
        className={`${radiusClass} ${className ?? ""}`}
        contentFit="cover"
      />
    );
  }
  const colors = gradientColors(seed);
  const icon = iconSize ?? Math.round(size * 0.42);
  return (
    <View
      style={{ width: size, height: size }}
      className={`${radiusClass} overflow-hidden ${className ?? ""}`}
    >
      <LinearGradient
        colors={colors}
        start={{ x: 0, y: 0 }}
        end={{ x: 1, y: 1 }}
        style={{ flex: 1, alignItems: "center", justifyContent: "center" }}
      >
        <Ionicons name="musical-notes" size={icon} color="#FFFFFF" />
      </LinearGradient>
    </View>
  );
}
