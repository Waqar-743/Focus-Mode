import "./App.css";
import { CircularTimer } from "./components/CircularTimer";
import { TimerControls } from "./components/TimerControls";
import { StatsDashboard } from "./components/StatsDashboard";
import { SessionTimeline } from "./components/SessionTimeline";
import { Settings } from "./components/Settings";
import { useTimer } from "./hooks/useTimer";
import { useTimerStore } from "./store/timerStore";
import { useState } from "react";

function App() {
  const { start, pause, reset } = useTimer();
  const blockingError = useTimerStore((s) => s.blockingError);
  const setBlockingError = useTimerStore((s) => s.setBlockingError);
  const [showSettings, setShowSettings] = useState(false);

  return (
    <main className="app">
      {/* Settings Button */}
      <button
        type="button"
        className="settings-btn"
        aria-label="Open settings"
        onClick={() => setShowSettings(true)}
      >
        ⚙
      </button>

      {/* Error Banner */}
      {blockingError && (
        <div className="error-banner" role="alert">
          <div className="error-banner__text">{blockingError}</div>
          <button
            type="button"
            className="error-banner__close"
            onClick={() => setBlockingError(null)}
            aria-label="Dismiss"
          >
            ✕
          </button>
        </div>
      )}

      {/* Main Content */}
      <div className="app__container">
        {/* Left Column - Timer & Controls */}
        <section className="timer-section">
          <CircularTimer />
          <TimerControls onStart={start} onPause={pause} onReset={reset} />
        </section>

        {/* Right Column - Dashboard & Timeline */}
        <section className="dashboard-section">
          <StatsDashboard />
          <SessionTimeline />
        </section>
      </div>

      {/* Settings Modal */}
      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
    </main>
  );
}

export default App;
