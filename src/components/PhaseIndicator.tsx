import { useTimerStore } from "../store/timerStore";
import type { Phase } from "../types/timer";

function label(phase: Phase) {
  switch (phase) {
    case "work":
      return "Work";
    case "short_break":
      return "Short Break";
    case "long_break":
      return "Long Break";
  }
}

export function PhaseIndicator() {
  const phase = useTimerStore((s) => s.phase);

  return (
    <div className="phase">
      <span className="phase__label">{label(phase)}</span>
    </div>
  );
}
