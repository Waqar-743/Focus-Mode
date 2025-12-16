import { useMemo } from "react";
import { useTimerStore } from "../store/timerStore";

export function Statistics() {
  const stats = useTimerStore((s) => s.todayStats);

  const goalProgress = useMemo(() => {
    if (!stats || stats.dailyGoal <= 0) return 0;
    return Math.min(1, Math.max(0, stats.workSessionsCompleted / stats.dailyGoal));
  }, [stats]);

  if (!stats) {
    return (
      <section className="stats" aria-label="Statistics">
        <div className="stats__header">
          <h2 className="stats__title">Today's Stats</h2>
        </div>
        <div className="stats__empty">Loading…</div>
      </section>
    );
  }

  return (
    <section className="stats" aria-label="Statistics">
      <div className="stats__header">
        <h2 className="stats__title">Today's Stats</h2>
        <div className="stats__date">{stats.date}</div>
      </div>

      <div className="stats__grid">
        <div className="stats__card">
          <div className="stats__key">Total Focus Time</div>
          <div className="stats__val">{stats.totalFocusMinutes} min</div>
        </div>
        <div className="stats__card">
          <div className="stats__key">Sessions</div>
          <div className="stats__val">
            {stats.workSessionsCompleted}/{stats.dailyGoal}
          </div>
        </div>
        <div className="stats__card">
          <div className="stats__key">Breaks Taken</div>
          <div className="stats__val">{stats.breaksTaken}</div>
        </div>
        <div className="stats__card">
          <div className="stats__key">Streak</div>
          <div className="stats__val">{stats.currentStreak}</div>
        </div>
        <div className="stats__card stats__card--wide">
          <div className="stats__key">Longest Streak</div>
          <div className="stats__val">{stats.longestStreak}</div>
        </div>
      </div>

      <div className="stats__progress" aria-label="Daily goal progress">
        <div className="stats__progressBar" style={{ width: `${goalProgress * 100}%` }} />
      </div>
    </section>
  );
}
