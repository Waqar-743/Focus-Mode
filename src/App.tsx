import "./App.css";
import { Controls } from "./components/Controls";
import { PhaseIndicator } from "./components/PhaseIndicator";
import { Settings } from "./components/Settings";
import { Statistics } from "./components/Statistics";
import { Timer } from "./components/Timer";
import { useTimer } from "./hooks/useTimer";
import { useTimerStore } from "./store/timerStore";
import { useState } from "react";

function App() {
  const { start, pause, reset } = useTimer();
  const phase = useTimerStore((s) => s.phase);
  const blockingError = useTimerStore((s) => s.blockingError);
  const setBlockingError = useTimerStore((s) => s.setBlockingError);
  const [showSettings, setShowSettings] = useState(false);

  return (
    <main className={`app app--${phase}`}>
      <header className="header">
        <div className="header__row">
          <h1 className="header__title">Focus Timer</h1>
          <button
            type="button"
            className="iconBtn"
            aria-label="Open settings"
            onClick={() => setShowSettings(true)}
          >
            ⚙
          </button>
        </div>
        <PhaseIndicator />
      </header>

      <div className="content">
        {blockingError && (
          <div className="banner" role="alert">
            <div className="banner__text">{blockingError}</div>
            <button
              type="button"
              className="banner__close"
              onClick={() => setBlockingError(null)}
              aria-label="Dismiss"
            >
              ✕
            </button>
          </div>
        )}
        <Timer />
        <Controls onStart={start} onPause={pause} onReset={reset} />
        {!showSettings && <Statistics />}
      </div>

      {showSettings && <Settings onClose={() => setShowSettings(false)} />}
    </main>
  );
}

export default App;
