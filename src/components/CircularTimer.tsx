import { useMemo } from "react";
import { useTimerStore } from "../store/timerStore";

function formatMMSS(totalSeconds: number) {
    const minutes = Math.floor(totalSeconds / 60);
    const seconds = totalSeconds % 60;
    return `${String(minutes).padStart(2, "0")}:${String(seconds).padStart(2, "0")}`;
}

export function CircularTimer() {
    const remainingSeconds = useTimerStore((s) => s.remainingSeconds);
    const totalSeconds = useTimerStore((s) => s.totalSeconds);

    const radius = 145;
    const circumference = 2 * Math.PI * radius;

    const offset = useMemo(() => {
        if (totalSeconds <= 0) return circumference;
        const progress = remainingSeconds / totalSeconds;
        return circumference - progress * circumference;
    }, [remainingSeconds, totalSeconds, circumference]);

    return (
        <div className="circular-timer">
            <svg className="circular-timer__svg">
                {/* Background Circle */}
                <circle
                    cx="190"
                    cy="190"
                    r={radius}
                    className="circular-timer__bg-circle"
                />
                {/* Progress Circle with coral glow */}
                <circle
                    cx="190"
                    cy="190"
                    r={radius}
                    className="circular-timer__progress-circle"
                    strokeDasharray={circumference}
                    strokeDashoffset={offset}
                />
            </svg>

            {/* Time Display */}
            <div className="circular-timer__time">
                <span className="circular-timer__time-text">
                    {formatMMSS(remainingSeconds)}
                </span>
            </div>
        </div>
    );
}
