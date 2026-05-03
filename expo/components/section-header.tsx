import { Text, View } from "react-native";

export function SectionHeader({
  title,
  subtitle,
}: {
  title: string;
  subtitle?: string;
}) {
  return (
    <View className="px-4 mb-3">
      <Text className="text-text-primary text-[22px] font-display">
        {title}
      </Text>
      {subtitle ? (
        <Text className="text-text-secondary text-[13px] mt-0.5 font-sans">
          {subtitle}
        </Text>
      ) : null}
    </View>
  );
}
