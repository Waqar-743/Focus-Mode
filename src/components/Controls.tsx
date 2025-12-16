import { useMemo } from "react";
import { useTimerStore } from "../store/timerStore";

export function Controls(props: {
  onStart: () => Promise<void>;
  onPause: () => Promise<void>;
  onReset: () => Promise<void>;
}) {
  const isRunning = useTimerStore((s) => s.isRunning);
  const remainingSeconds = useTimerStore((s) => s.remainingSeconds);
  const totalSeconds = useTimerStore((s) => s.totalSeconds);

  const startLabel = useMemo(() => {
    if (isRunning) return "Start";
    if (remainingSeconds !== totalSeconds) return "Resume";
    return "Start";
  }, [isRunning, remainingSeconds, totalSeconds]);

  return (
    <section className="controls" aria-label="Timer controls">
      <button
        className="btn btn--primary"
        type="button"
        disabled={isRunning}
        onClick={() => void props.onStart()}
      >
        {startLabel}
      </button>

      <button
        className="btn btn--secondary"
        type="button"
        disabled={!isRunning}
        onClick={() => void props.onPause()}
      >
        Pause
      </button>

      <button
        className="btn btn--ghost"
        type="button"
        onClick={() => void props.onReset()}
      >
        Reset
      </button>
    </section>
  );
}
