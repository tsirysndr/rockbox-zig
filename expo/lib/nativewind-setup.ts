import { BlurView } from "expo-blur";
import { Image } from "expo-image";
import { LinearGradient } from "expo-linear-gradient";
import { cssInterop } from "nativewind";
import { SafeAreaView } from "react-native-safe-area-context";

cssInterop(Image, { className: "style" });
cssInterop(BlurView, { className: "style" });
cssInterop(LinearGradient, { className: "style" });
cssInterop(SafeAreaView, { className: "style" });
