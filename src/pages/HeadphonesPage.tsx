import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { useAudioDevices } from "../lib/useAudioDevices";

type Channel = "left" | "right" | "both";

export default function HeadphonesPage() {
  const { devices, error: deviceError, loading, refresh } = useAudioDevices("audiooutput");
  const [selectedId, setSelectedId] = useState<string>("");
  const [playing, setPlaying] = useState<Channel | null>(null);
  const [playError, setPlayError] = useState<string | null>(null);

  useEffect(() => {
    if (!selectedId && devices.length > 0) {
      setSelectedId(devices[0].deviceId);
    }
  }, [devices, selectedId]);

  useEffect(() => {
    return () => {
      invoke("stop_test_tone").catch(() => {});
    };
  }, []);

  async function playChannel(channel: Channel) {
    setPlayError(null);
    try {
      await invoke("play_test_tone", { deviceId: selectedId, channel });
      setPlaying(channel);
    } catch (err) {
      setPlayError(err instanceof Error ? err.message : String(err));
    }
  }

  async function stopTone() {
    try {
      await invoke("stop_test_tone");
    } catch (err) {
      setPlayError(err instanceof Error ? err.message : String(err));
    } finally {
      setPlaying(null);
    }
  }

  return (
    <div className="page">
      <h1>Headphones / Speakers Test</h1>
      <p className="page-subtitle">
        Pick your output device, then test left, right, and both channels to confirm
        your audio is wired and working correctly.
      </p>

      <div className="field">
        <label htmlFor="output-select">Output device</label>
        <div className="row-inline">
          <select
            id="output-select"
            value={selectedId}
            onChange={(e) => setSelectedId(e.target.value)}
            disabled={loading || devices.length === 0}
          >
            {devices.length === 0 && <option value="">No outputs found</option>}
            {devices.map((d) => (
              <option key={d.deviceId} value={d.deviceId}>
                {d.label}
              </option>
            ))}
          </select>
          <button type="button" onClick={refresh} disabled={loading}>
            Refresh
          </button>
        </div>
      </div>

      {(deviceError || playError) && (
        <div className="error-box">{deviceError || playError}</div>
      )}

      <div className="channel-buttons">
        <button
          type="button"
          className={playing === "left" ? "active" : ""}
          onClick={() => playChannel("left")}
        >
          Test Left
        </button>
        <button
          type="button"
          className={playing === "both" ? "active" : ""}
          onClick={() => playChannel("both")}
        >
          Test Both
        </button>
        <button
          type="button"
          className={playing === "right" ? "active" : ""}
          onClick={() => playChannel("right")}
        >
          Test Right
        </button>
      </div>

      <div className="record-section">
        <button type="button" onClick={stopTone} disabled={!playing}>
          Stop
        </button>
      </div>
    </div>
  );
}
