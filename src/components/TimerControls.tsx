import { useTimerStore } from "../store/timerStore";

interface TimerControlsProps {
    onStart: () => Promise<void>;
    onPause: () => Promise<void>;
    onReset: () => Promise<void>;
}

export function TimerControls({ onStart, onPause, onReset }: TimerControlsProps) {
    const isRunning = useTimerStore((s) => s.isRunning);

    const handleStartPause = () => {
        if (isRunning) {
            void onPause();
        } else {
            void onStart();
        }
    };

    return (
        <div className="timer-controls">
            {/* Start/Pause Button */}
            <button
                type="button"
                className="timer-controls__btn timer-controls__btn--primary"
                onClick={handleStartPause}
            >
                <div className="timer-controls__icon-group">
                    {/* Play Icon */}
                    <svg width="16" height="18" viewBox="0 0 24 24" fill="currentColor">
                        <path d="M8 5v14l11-7z" />
                    </svg>

                    <div className="timer-controls__icon-divider" />

                    {/* Pause Icon */}
                    <div className="timer-controls__pause-bars">
                        <div className="timer-controls__pause-bar" />
                        <div className="timer-controls__pause-bar" />
                    </div>
                </div>
                <span>Start/Pause</span>
            </button>

            {/* Reset Button */}
            <button
                type="button"
                className="timer-controls__btn timer-controls__btn--secondary"
                onClick={() => void onReset()}
            >
                Reset
            </button>
        </div>
    );
}
