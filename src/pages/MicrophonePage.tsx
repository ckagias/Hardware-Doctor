import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import { useAudioDevices } from "../lib/useAudioDevices";

type RecordState = "idle" | "recording" | "playing";

export default function MicrophonePage() {
  const { devices, error: deviceError, loading, refresh } = useAudioDevices("audioinput");
  const [selectedId, setSelectedId] = useState<string>("");
  const [level, setLevel] = useState(0);
  const [peakLevel, setPeakLevel] = useState(0);
  const [monitorError, setMonitorError] = useState<string | null>(null);
  const [recordState, setRecordState] = useState<RecordState>("idle");
  const [playbackSrc, setPlaybackSrc] = useState<string | null>(null);

  const audioElRef = useRef<HTMLAudioElement | null>(null);

  useEffect(() => {
    if (!selectedId && devices.length > 0) {
      setSelectedId(devices[0].deviceId);
    }
  }, [devices, selectedId]);

  useEffect(() => {
    let cancelled = false;
    let unlisten: (() => void) | null = null;

    async function startMonitoring() {
      setMonitorError(null);
      setPeakLevel(0);
      if (!selectedId) return;

      try {
        unlisten = await listen<{ level: number }>("mic-level", (event) => {
          const pct = Math.round(event.payload.level);
          setLevel(pct);
          setPeakLevel((prev) => (pct > prev ? pct : Math.max(0, prev - 1)));
        });
        if (cancelled) return;
        await invoke("start_mic_monitor", { deviceId: selectedId });
      } catch (err) {
        if (!cancelled) {
          setMonitorError(err instanceof Error ? err.message : String(err));
        }
      }
    }

    startMonitoring();
    return () => {
      cancelled = true;
      invoke("stop_mic_monitor").catch(() => {});
      if (unlisten) unlisten();
      setLevel(0);
    };
  }, [selectedId]);

  async function startRecording() {
    if (!selectedId) return;
    setRecordState("recording");
    try {
      const base64Wav = await invoke<string>("record_mic_clip", {
        deviceId: selectedId,
        durationMs: 4000,
      });
      const src = `data:audio/wav;base64,${base64Wav}`;
      setPlaybackSrc(src);
      setRecordState("playing");
      if (audioElRef.current) {
        audioElRef.current.src = src;
        audioElRef.current.play();
      }
    } catch (err) {
      setMonitorError(err instanceof Error ? err.message : String(err));
      setRecordState("idle");
    }
  }

  return (
    <div className="page">
      <h1>Microphone Test</h1>
      <p className="page-subtitle">
        Pick your mic below, speak, and watch the level meter. Use "Record &amp; Play
        Back" to hear exactly what your mic captures.
      </p>

      <div className="field">
        <label htmlFor="mic-select">Input device</label>
        <div className="row-inline">
          <select
            id="mic-select"
            value={selectedId}
            onChange={(e) => setSelectedId(e.target.value)}
            disabled={loading || devices.length === 0}
          >
            {devices.length === 0 && <option value="">No microphones found</option>}
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

      {(deviceError || monitorError) && (
        <div className="error-box">{deviceError || monitorError}</div>
      )}

      <div className="meter-section">
        <div className="meter-label">
          <span>Input Level</span>
          <span>{level}%</span>
        </div>
        <div className="meter-track">
          <div
            className="meter-fill"
            style={{
              width: `${level}%`,
              background:
                level > 85 ? "#e5484d" : level > 60 ? "#f5b800" : "#30a46c",
            }}
          />
          <div className="meter-peak" style={{ left: `${peakLevel}%` }} />
        </div>
      </div>

      <div className="record-section">
        <button
          type="button"
          onClick={startRecording}
          disabled={!selectedId || recordState === "recording"}
        >
          {recordState === "recording" ? "Recording... (4s)" : "Record & Play Back"}
        </button>
        <audio
          ref={audioElRef}
          controls
          onEnded={() => setRecordState("idle")}
          style={{ display: playbackSrc ? "block" : "none" }}
        />
      </div>
    </div>
  );
}
