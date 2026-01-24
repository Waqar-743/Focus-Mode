import { useTimerStore } from "../store/timerStore";

interface StatCardProps {
    label: string;
    value: string | number;
    unit?: string;
}

function StatCard({ label, value, unit }: StatCardProps) {
    return (
        <div className="stat-card">
            <span className="stat-card__label">{label}</span>
            <div className="stat-card__value">
                {value}
                {unit && <span className="stat-card__unit">{unit}</span>}
            </div>
        </div>
    );
}

export function StatsDashboard() {
    const stats = useTimerStore((s) => s.todayStats);
    const dailyGoal = useTimerStore((s) => s.dailyGoal);
    const completedWorkSessions = useTimerStore((s) => s.completedWorkSessions);

    // Use stats if available, otherwise use store values
    const totalFocusMinutes = stats?.totalFocusMinutes ?? 0;
    const sessionsCompleted = stats?.workSessionsCompleted ?? completedWorkSessions;
    const breaksTaken = stats?.breaksTaken ?? 0;
    const currentStreak = stats?.currentStreak ?? 0;
    const goal = stats?.dailyGoal ?? dailyGoal;

    return (
        <div className="dashboard-section">
            <h2 className="dashboard__title">Dashboard</h2>
            <h3 className="dashboard__subtitle">Today's Performance</h3>

            <div className="stats-grid">
                <StatCard
                    label="Total Focus Time"
                    value={totalFocusMinutes}
                    unit="min"
                />
                <StatCard
                    label="Sessions"
                    value={`${sessionsCompleted}/${goal}`}
                />
                <StatCard
                    label="Breaks"
                    value={breaksTaken}
                />
                <StatCard
                    label="Streak"
                    value={currentStreak}
                />
            </div>
        </div>
    );
}
