export type Phase = "work" | "short_break" | "long_break";

export interface TimerState {
  phase: Phase;
  totalSeconds: number;
  remainingSeconds: number;
  isRunning: boolean;
  completedWorkSessions: number;
  longBreakAfter: number;
  dailyGoal: number;
}

export interface PhaseCompleteEvent {
  from: Phase;
  to: Phase;
  completedSeconds?: number;
}

export interface Settings {
  workMinutes: number;
  shortBreakMinutes: number;
  longBreakMinutes: number;
  longBreakAfter: number;
  dailyGoal: number;
}

export interface TodayStatistics {
  date: string;
  totalFocusMinutes: number;
  workSessionsCompleted: number;
  breaksTaken: number;
  currentStreak: number;
  longestStreak: number;
  dailyGoal: number;
}
