import { invoke } from "@tauri-apps/api/core";
import { useCallback, useEffect, useState } from "react";

export interface AudioDeviceInfo {
  deviceId: string;
  label: string;
}

interface RustDeviceInfo {
  id: string;
  label: string;
}

export function useAudioDevices(kind: "audioinput" | "audiooutput") {
  const [devices, setDevices] = useState<AudioDeviceInfo[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);

  const refresh = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const command = kind === "audioinput" ? "list_input_devices" : "list_output_devices";
      const result = await invoke<RustDeviceInfo[]>(command);
      setDevices(result.map((d) => ({ deviceId: d.id, label: d.label })));
    } catch (err) {
      setError(err instanceof Error ? err.message : String(err));
    } finally {
      setLoading(false);
    }
  }, [kind]);

  useEffect(() => {
    refresh();
  }, [refresh]);

  return { devices, error, loading, refresh };
}
