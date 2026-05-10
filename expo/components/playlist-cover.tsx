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

const HERO_GRADIENTS: [string, string][] = [
  ["#FF00FF", "#00FFCC"],
  ["#FF00AA", "#00FF87"],
  ["#FF00FF", "#39FF14"],
  ["#00FFFF", "#FF00FF"],
  ["#FF0080", "#00FFCC"],
  ["#CC00FF", "#00FF87"],
  ["#FF00FF", "#60EFFF"],
  ["#FF00CC", "#00FF41"],
  ["#FF00FF", "#00FFFF"],
  ["#FF0099", "#39FF14"],
  ["#FF00CC", "#00FFEF"],
  ["#FF00FF", "#00FF41"],
  ["#CC00FF", "#00FFCC"],
  ["#FF0080", "#60EFFF"],
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

export function heroGradientColors(seed?: string): [string, string] {
  return HERO_GRADIENTS[hashSeed(seed) % HERO_GRADIENTS.length];
}

const ARTIST_GRADIENTS: [string, string][] = [
  ["#1C1C2E", "#4A3F8F"],
  ["#0D1B2A", "#1B6CA8"],
  ["#12192C", "#2E7D6E"],
  ["#1E0B33", "#6B3FA0"],
  ["#0B2A2A", "#1A7A6E"],
  ["#1A1226", "#5C3472"],
  ["#0E2233", "#1E6B9A"],
  ["#1A0A2E", "#7B3F8C"],
  ["#0C1F1A", "#1E6B52"],
  ["#1C1030", "#5A2D82"],
  ["#0A1E2E", "#1A5C8A"],
  ["#1A1530", "#4A2D7A"],
  ["#0E2218", "#1E6640"],
  ["#1E1228", "#6B3D8C"],
];

export function artistGradientColors(seed?: string): [string, string] {
  return ARTIST_GRADIENTS[hashSeed(seed) % ARTIST_GRADIENTS.length];
}

function hexLuminance(hex: string): number {
  const r = parseInt(hex.slice(1, 3), 16) / 255;
  const g = parseInt(hex.slice(3, 5), 16) / 255;
  const b = parseInt(hex.slice(5, 7), 16) / 255;
  const lin = (c: number) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4);
  return 0.2126 * lin(r) + 0.7152 * lin(g) + 0.0722 * lin(b);
}

/** Returns "#FFFFFF" or "#000000" for maximum contrast against the gradient. */
export function gradientIconColor(colors: [string, string]): string {
  const lum = (hexLuminance(colors[0]) + hexLuminance(colors[1])) / 2;
  return lum > 0.35 ? "#000000" : "#FFFFFF";
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
