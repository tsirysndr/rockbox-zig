import {
  IconBluetooth,
  IconCast,
  IconCheck,
  IconDeviceMobile,
  IconHeadphones,
  IconMusic,
  IconVolume,
  type Icon,
} from "@tabler/icons-react-native";
import { Modal, Pressable, ScrollView, Text, View } from "react-native";
import Svg, { Path, Rect } from "react-native-svg";

import { Colors } from "@/constants/theme";
import { useIsConnected } from "@/lib/connection";
import {
  useConnectDevice,
  useDisconnectDevice,
  useOutputDevices,
} from "@/lib/queries";

export type DeviceItem = {
  id?: string;
  name?: string;
  ip?: string;
  port?: number;
  service?: string;
  app?: string;
  is_connected?: boolean;
  is_current_device?: boolean;
};

export type DeviceIconSpec =
  | { kind: "tabler"; component: Icon }
  | { kind: "airplay" };

const DEVICE_ICON: Record<string, DeviceIconSpec> = {
  googlecast: { kind: "tabler", component: IconCast },
  chromecast: { kind: "tabler", component: IconCast },
  airplay: { kind: "airplay" },
  snapcast: { kind: "tabler", component: IconVolume },
  upnp: { kind: "tabler", component: IconMusic },
  bluetooth: { kind: "tabler", component: IconBluetooth },
  builtin: { kind: "tabler", component: IconDeviceMobile },
};

const DEFAULT_ICON: DeviceIconSpec = {
  kind: "tabler",
  component: IconHeadphones,
};

export function iconFor(svc?: string): DeviceIconSpec {
  if (!svc) return DEFAULT_ICON;
  const key = svc.toLowerCase();
  for (const k of Object.keys(DEVICE_ICON)) {
    if (key.includes(k)) return DEVICE_ICON[k];
  }
  return DEFAULT_ICON;
}

/** AirPlay glyph — Tabler doesn't ship one, so we render the canonical
 *  rectangle-over-triangle SVG inline. */
function AirplayIcon({ size, color }: { size: number; color: string }) {
  return (
    <Svg width={size} height={size} viewBox="0 0 24 24" fill="none">
      <Rect
        x={3}
        y={4}
        width={18}
        height={13}
        rx={2}
        ry={2}
        stroke={color}
        strokeWidth={2}
        strokeLinejoin="round"
      />
      <Path
        d="M8 21l4-5 4 5z"
        fill={color}
        stroke={color}
        strokeWidth={1.5}
        strokeLinejoin="round"
      />
    </Svg>
  );
}

export function DeviceIcon({
  spec,
  size,
  color,
}: {
  spec: DeviceIconSpec;
  size: number;
  color: string;
}) {
  if (spec.kind === "airplay") {
    return <AirplayIcon size={size} color={color} />;
  }
  const Component = spec.component;
  return <Component size={size} color={color} strokeWidth={1.75} />;
}

/** Small label for the player-bar trigger. Shows the active device name. */
export function useCurrentDeviceLabel(): {
  name: string;
  icon: DeviceIconSpec;
} {
  const isConnected = useIsConnected();
  const { data } = useOutputDevices<DeviceItem[]>({ enabled: isConnected });
  const list: DeviceItem[] = Array.isArray(data) ? data : [];
  const current = list.find((d) => d.is_current_device);
  return {
    name: current?.name ?? "This Phone",
    icon: iconFor(current?.service),
  };
}

export function DevicePickerSheet({
  visible,
  onClose,
}: {
  visible: boolean;
  onClose: () => void;
}) {
  const isConnected = useIsConnected();
  const { data, isLoading } = useOutputDevices<DeviceItem[]>({
    enabled: isConnected && visible,
  });
  const list: DeviceItem[] = Array.isArray(data) ? data : [];
  const connect = useConnectDevice();
  const disconnect = useDisconnectDevice();

  const onPick = (d: DeviceItem) => {
    if (!d.id) return;
    if (d.is_current_device || d.is_connected) {
      disconnect.mutate(d.id);
    } else {
      connect.mutate(d.id);
    }
    onClose();
  };

  return (
    <Modal
      visible={visible}
      transparent
      animationType="slide"
      onRequestClose={onClose}
    >
      <Pressable onPress={onClose} className="flex-1 bg-black/55">
        <Pressable
          onPress={(e) => e.stopPropagation()}
          className="mt-auto bg-bg-elevated rounded-t-2xl pt-2 pb-7"
          style={{ maxHeight: "70%" }}
        >
          <View className="self-center w-10 h-1 rounded-sm bg-border my-2" />
          <Text className="text-text-primary text-base font-bold text-center py-2 font-sans">
            Output devices
          </Text>
          <ScrollView>
            {!isConnected ? (
              <Text className="text-text-secondary text-sm text-center py-6 font-sans">
                Connect to a server first.
              </Text>
            ) : isLoading && list.length === 0 ? (
              <Text className="text-text-secondary text-sm text-center py-6 font-sans">
                Loading devices…
              </Text>
            ) : list.length === 0 ? (
              <Text className="text-text-secondary text-sm text-center py-6 font-sans">
                No output devices found.
              </Text>
            ) : (
              list.map((d, idx) => {
                const active = d.is_current_device === true;
                const subtitle = [
                  d.service?.toUpperCase(),
                  d.ip ? `${d.ip}${d.port ? `:${d.port}` : ""}` : null,
                ]
                  .filter(Boolean)
                  .join(" • ");
                return (
                  <Pressable
                    key={d.id ?? idx}
                    onPress={() => onPick(d)}
                    android_ripple={{ color: Colors.bgHover }}
                    className="flex-row items-center gap-3 px-5 py-3 active:bg-bg-hover"
                  >
                    <View
                      className={`w-10 h-10 rounded-full items-center justify-center ${active ? "bg-accent" : "bg-bg-card"}`}
                    >
                      <DeviceIcon
                        spec={iconFor(d.service)}
                        size={18}
                        color={active ? "#FFFFFF" : Colors.textPrimary}
                      />
                    </View>
                    <View className="flex-1 min-w-0">
                      <Text
                        numberOfLines={1}
                        className={`text-[14px] font-semibold font-sans ${active ? "text-accent" : "text-text-primary"}`}
                      >
                        {d.name?.trim() || "(unnamed)"}
                      </Text>
                      {subtitle ? (
                        <Text
                          numberOfLines={1}
                          className="text-text-secondary text-[12px] mt-0.5 font-mono"
                        >
                          {subtitle}
                        </Text>
                      ) : null}
                    </View>
                    {active ? (
                      <IconCheck
                        size={20}
                        color={Colors.accent}
                        strokeWidth={2}
                      />
                    ) : null}
                  </Pressable>
                );
              })
            )}
          </ScrollView>
        </Pressable>
      </Pressable>
    </Modal>
  );
}
