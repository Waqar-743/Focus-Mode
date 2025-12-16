import { create } from "zustand";
import type { Settings, TodayStatistics, TimerState } from "../types/timer";

type TimerStore = TimerState & {
  settings: Settings | null;
  todayStats: TodayStatistics | null;
  blockingError: string | null;
  setFromBackend: (state: TimerState) => void;
  setSettings: (settings: Settings) => void;
  setTodayStats: (stats: TodayStatistics) => void;
  setBlockingError: (message: string | null) => void;
};

const defaultState: TimerState = {
  phase: "work",
  totalSeconds: 25 * 60,
  remainingSeconds: 25 * 60,
  isRunning: false,
  completedWorkSessions: 0,
  longBreakAfter: 4,
  dailyGoal: 8,
};

export const useTimerStore = create<TimerStore>((set) => ({
  ...defaultState,
  settings: null,
  todayStats: null,
  blockingError: null,
  setFromBackend: (state) => set(state),
  setSettings: (settings) => set({ settings }),
  setTodayStats: (todayStats) => set({ todayStats }),
  setBlockingError: (blockingError) => set({ blockingError }),
}));
