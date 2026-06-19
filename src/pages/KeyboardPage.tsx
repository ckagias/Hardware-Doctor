import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useState } from "react";
import type { KeyboardLayout } from "../lib/keyboardLayout";

type KeyboardSnapshot = {
  pressed: string[];
  tested: string[];
  testedCount: number;
  totalCount: number;
};

const EMPTY_LAYOUT: KeyboardLayout = { main: [], numpad: [] };

export default function KeyboardPage() {
  const [layout, setLayout] = useState<KeyboardLayout>(EMPTY_LAYOUT);
  const [pressed, setPressed] = useState<Set<string>>(new Set());
  const [tested, setTested] = useState<Set<string>>(new Set());
  const [testedCount, setTestedCount] = useState(0);
  const [totalCount, setTotalCount] = useState(0);
  const [error, setError] = useState<string | null>(null);

  function applySnapshot(snapshot: KeyboardSnapshot) {
    setPressed(new Set(snapshot.pressed));
    setTested(new Set(snapshot.tested));
    setTestedCount(snapshot.testedCount);
    setTotalCount(snapshot.totalCount);
  }

  // Rust owns the pressed/tested state; this page just forwards key codes and renders the result
  useEffect(() => {
    let unlisten: (() => void) | undefined;
    let cancelled = false;

    (async () => {
      try {
        unlisten = await listen<KeyboardSnapshot>("keyboard-state", (e) => {
          applySnapshot(e.payload);
        });
        if (cancelled) return;

        const [fetchedLayout, snapshot] = await Promise.all([
          invoke<KeyboardLayout>("keyboard_layout"),
          invoke<KeyboardSnapshot>("keyboard_snapshot"),
        ]);
        if (cancelled) return;
        setLayout(fetchedLayout);
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
    const allKeys = new Set([
      ...layout.main.map((k) => k.code),
      ...layout.numpad.map((k) => k.code),
    ]);

    function onKeyDown(e: KeyboardEvent) {
      if (!allKeys.has(e.code)) return;
      e.preventDefault();
      invoke("key_down", { code: e.code }).catch(() => {});
    }

    function onKeyUp(e: KeyboardEvent) {
      if (!allKeys.has(e.code)) return;
      e.preventDefault();
      invoke("key_up", { code: e.code }).catch(() => {});
    }

    function onBlur() {
      invoke("clear_pressed").catch(() => {});
    }

    window.addEventListener("keydown", onKeyDown);
    window.addEventListener("keyup", onKeyUp);
    window.addEventListener("blur", onBlur);
    return () => {
      window.removeEventListener("keydown", onKeyDown);
      window.removeEventListener("keyup", onKeyUp);
      window.removeEventListener("blur", onBlur);
    };
  }, [layout]);

  function resetTested() {
    invoke("reset_tested").catch((err) => {
      setError(err instanceof Error ? err.message : String(err));
    });
  }

  function keyClassNames(code: string) {
    const isPressed = pressed.has(code);
    const isTested = tested.has(code);
    return [
      "kb-key",
      isPressed ? "kb-key-pressed" : "",
      isTested && !isPressed ? "kb-key-tested" : "",
    ]
      .filter(Boolean)
      .join(" ");
  }

  // Each grid unit is split into 4 columns so keys can be offset by quarter-widths (e.g. col: 7.5)
  const UNIT = 4;
  function gridStyle(keyDef: { col: number; row: number; colSpan?: number; rowSpan?: number }) {
    return {
      gridColumn: `${Math.round((keyDef.col - 1) * UNIT) + 1} / span ${Math.round((keyDef.colSpan ?? 1) * UNIT)}`,
      gridRow: `${keyDef.row + 1} / span ${keyDef.rowSpan ?? 1}`,
      // tall keys (NumpadAdd, NumpadEnter) fill their full row span
      ...(keyDef.rowSpan && keyDef.rowSpan > 1 ? { height: "100%" } : {}),
    };
  }

  return (
    <div className="page page-wide keyboard-page">
      <h1>Keyboard Test</h1>
      <p className="page-subtitle">
        Click into this window, then press keys on your physical keyboard. Each key
        lights up while held, and turns green once it's been tested at least once.
      </p>

      {error && <div className="error-box">{error}</div>}

      <div className="kb-progress">
        <span>
          {testedCount} / {totalCount} keys tested
        </span>
        <button type="button" onClick={resetTested}>
          Reset
        </button>
      </div>

      <div className="keyboard" tabIndex={0}>
        {[...layout.main, ...layout.numpad].map((keyDef) => (
          <div
            key={keyDef.code}
            className={keyClassNames(keyDef.code)}
            style={gridStyle(keyDef)}
          >
            {keyDef.label}
          </div>
        ))}
      </div>
    </div>
  );
}
