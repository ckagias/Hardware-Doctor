import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";

type MouseSnapshot = {
  pressed: string[];
  tested: string[];
  testedCount: number;
  totalCount: number;
};

// Maps MouseEvent.button to the stable codes the backend tracks
const BUTTON_CODES: Record<number, string> = {
  0: "Left",
  1: "Middle",
  2: "Right",
  3: "Back",
  4: "Forward",
};

// Scroll events have no native "up" counterpart, so the pressed flash is cleared by hand
const SCROLL_FLASH_MS = 150;

export default function MousePage() {
  const [pressed, setPressed] = useState<Set<string>>(new Set());
  const [tested, setTested] = useState<Set<string>>(new Set());
  const [testedCount, setTestedCount] = useState(0);
  const [totalCount, setTotalCount] = useState(0);
  const [error, setError] = useState<string | null>(null);

  function applySnapshot(snapshot: MouseSnapshot) {
    setPressed(new Set(snapshot.pressed));
    setTested(new Set(snapshot.tested));
    setTestedCount(snapshot.testedCount);
    setTotalCount(snapshot.totalCount);
  }

  // Rust owns the pressed/tested state; this page just forwards mouse events and renders the result
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    (async () => {
      try {
        unlisten = await listen<MouseSnapshot>("mouse-state", (e) => {
          applySnapshot(e.payload);
        });
        if (cancelled) return;

        const snapshot = await invoke<MouseSnapshot>("mouse_snapshot");
        if (cancelled) return;
        applySnapshot(snapshot);
      } catch (err) {
        if (!cancelled) {
          setError(err instanceof Error ? err.message : String(err));
        }
      }
    })();

    return () => {
      cancelled = true;
      unlisten?.();
    };
  }, []);

  useEffect(() => {
    function onMouseDown(e: MouseEvent) {
      const code = BUTTON_CODES[e.button];
      if (!code) return;
      if (e.button === 2 || e.button === 3 || e.button === 4) {
        e.preventDefault();
      }
      invoke("button_down", { code }).catch(() => {});
    }

    function onMouseUp(e: MouseEvent) {
      const code = BUTTON_CODES[e.button];
      if (!code) return;
      invoke("button_up", { code }).catch(() => {});
    }

    function onContextMenu(e: MouseEvent) {
      e.preventDefault();
    }

    function onWheel(e: WheelEvent) {
      e.preventDefault();
      const direction = e.deltaY < 0 ? "up" : "down";
      const code = direction === "up" ? "ScrollUp" : "ScrollDown";
      invoke("scroll", { direction }).catch(() => {});
      setTimeout(() => {
        invoke("button_up", { code }).catch(() => {});
      }, SCROLL_FLASH_MS);
    }

    function onBlur() {
      invoke("mouse_clear_pressed").catch(() => {});
    }

    window.addEventListener("mousedown", onMouseDown);
    window.addEventListener("mouseup", onMouseUp);
    window.addEventListener("contextmenu", onContextMenu);
    window.addEventListener("wheel", onWheel, { passive: false });
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("mousedown", onMouseDown);
      window.removeEventListener("mouseup", onMouseUp);
      window.removeEventListener("contextmenu", onContextMenu);
      window.removeEventListener("wheel", onWheel);
      window.removeEventListener("blur", onBlur);
    };
  }, []);

  function resetTested() {
    invoke("mouse_reset_tested").catch((err) => {
      setError(err instanceof Error ? err.message : String(err));
    });
  }

  function zoneClassNames(code: string, base: string) {
    const isPressed = pressed.has(code);
    const isTested = tested.has(code);
    return [
      base,
      isPressed ? "mouse-zone-pressed" : "",
      isTested && !isPressed ? "mouse-zone-tested" : "",
    ]
      .filter(Boolean)
      .join(" ");
  }

  return (
    <div className="page">
      <h1>Mouse Test</h1>
      <p className="page-subtitle">
        Click into this window, then click each mouse button, press the side buttons,
        and scroll the wheel up and down. Each input lights up while active, and turns
        green once it's been tested at least once.
      </p>

      {error && <div className="error-box">{error}</div>}

      <div className="kb-progress">
        <span>
          {testedCount} / {totalCount} inputs tested
        </span>
        <button type="button" onClick={resetTested}>
          Reset
        </button>
      </div>

      <div className="mouse-diagram">
        <div className="mouse-outline">
          <div className="mouse-side-buttons">
            <div className={zoneClassNames("Back", "mouse-zone mouse-side-button")} />
            <div className={zoneClassNames("Forward", "mouse-zone mouse-side-button")} />
          </div>

          <div className="mouse-body">
            <div className="mouse-top-row">
              <div className={zoneClassNames("Left", "mouse-zone mouse-click-left")}>
                Left
              </div>
              <div className="mouse-center-column">
                <div
                  className={zoneClassNames("ScrollUp", "mouse-zone mouse-scroll-indicator")}
                >
                  ▲
                </div>
                <div className={zoneClassNames("Middle", "mouse-zone mouse-wheel")} />
                <div
                  className={zoneClassNames("ScrollDown", "mouse-zone mouse-scroll-indicator")}
                >
                  ▼
                </div>
              </div>
              <div className={zoneClassNames("Right", "mouse-zone mouse-click-right")}>
                Right
              </div>
            </div>
            <div className="mouse-bottom" />
          </div>
        </div>
      </div>
    </div>
  );
}
