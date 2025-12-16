import { useMemo } from "react";
import { useTimerStore } from "../store/timerStore";

function formatMMSS(totalSeconds: number) {
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export function Timer() {
  const phase = useTimerStore((s) => s.phase);
  const remainingSeconds = useTimerStore((s) => s.remainingSeconds);
  const totalSeconds = useTimerStore((s) => s.totalSeconds);
  const completedWorkSessions = useTimerStore((s) => s.completedWorkSessions);
  const longBreakAfter = useTimerStore((s) => s.longBreakAfter);
  const dailyGoal = useTimerStore((s) => s.dailyGoal);

  const progress = useMemo(() => {
    if (totalSeconds <= 0) return 0;
    return 1 - Math.min(1, Math.max(0, remainingSeconds / totalSeconds));
  }, [remainingSeconds, totalSeconds]);

  const cycleIndex = longBreakAfter > 0 ? (completedWorkSessions % longBreakAfter) + 1 : 1;
  const sessionIndex = Math.min(
    dailyGoal,
    Math.max(1, completedWorkSessions + (phase === "work" ? 1 : 0)),
  );

  return (
    <section className="timer">
      <div className="timer__time" aria-label="Countdown timer">
        {formatMMSS(remainingSeconds)}
      </div>

      <div className="timer__meta">
        <div className="timer__metaRow">
          <span className="timer__metaKey">Session</span>
          <span className="timer__metaVal">
            {sessionIndex}/{dailyGoal}
          </span>
        </div>
        <div className="timer__metaRow">
          <span className="timer__metaKey">Cycle</span>
          <span className="timer__metaVal">
            {cycleIndex}/{longBreakAfter}
          </span>
        </div>
      </div>

      <div className="progress" aria-label="Phase progress">
        <div className="progress__bar" style={{ width: `${progress * 100}%` }} />
      </div>
    </section>
  );
}
