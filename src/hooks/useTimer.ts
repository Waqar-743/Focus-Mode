import { useEffect, useMemo } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from "@tauri-apps/plugin-notification";
import { useTimerStore } from "../store/timerStore";
import type {
  Phase,
  PhaseCompleteEvent,
  Settings,
  TimerState,
  TodayStatistics,
} from "../types/timer";

function phaseLabel(phase: Phase): string {
  switch (phase) {
    case "work":
      return "Work";
    case "short_break":
      return "Short Break";
    case "long_break":
      return "Long Break";
  }
}

function notificationBody(from: Phase, to: Phase): string {
  // Keep these messages consistent with the project brief.
  if (from === "work" && to === "short_break") return "Time to get back to work! 💪";
  if (from === "short_break" && to === "work") return "Take a quick 5-minute rest! ☕";
  if (from === "long_break" && to === "work") return "Enjoy your 15-minute break! 🌟";
  return `${phaseLabel(from)} → ${phaseLabel(to)}`;
}

export function useTimer() {
  const setFromBackend = useTimerStore((s) => s.setFromBackend);
  const setSettings = useTimerStore((s) => s.setSettings);
  const setTodayStats = useTimerStore((s) => s.setTodayStats);
  const setBlockingError = useTimerStore((s) => s.setBlockingError);

  useEffect(() => {
    let mounted = true;

    (async () => {
      try {
        const granted = await isPermissionGranted();
        if (!granted) await requestPermission();
      } catch {
        // Notifications are optional; ignore if unavailable.
      }

      try {
        const state = (await invoke("initialize_timer")) as TimerState;
        if (mounted) setFromBackend(state);
      } catch {
        // If initialization fails, the app can still render with defaults.
      }

      try {
        const settings = (await invoke("get_settings")) as Settings;
        if (mounted) setSettings(settings);
      } catch {
        // Ignore if settings can't be loaded.
      }

      try {
        const stats = (await invoke("get_today_statistics")) as TodayStatistics;
        if (mounted) setTodayStats(stats);
      } catch {
        // Ignore if stats can't be loaded.
      }
    })();

    return () => {
      mounted = false;
    };
  }, [setFromBackend, setSettings, setTodayStats]);

  useEffect(() => {
    const unlistenState = listen<TimerState>("timer-state", (event) => {
      setFromBackend(event.payload);
    });

    const unlistenPhase = listen<PhaseCompleteEvent>("phase-complete", (event) => {
      const { from, to } = event.payload;
      try {
        sendNotification({
          title: "Phase Complete",
          body: notificationBody(from, to),
        });
      } catch {
        // Ignore if notifications fail.
      }
    });

    const unlistenTodayStats = listen<TodayStatistics>(
      "today-statistics",
      (event) => {
        setTodayStats(event.payload);
      },
    );

    const unlistenBlockingError = listen<{ message: string }>(
      "blocking-error",
      (event) => {
        setBlockingError(event.payload.message);
      },
    );

    return () => {
      void unlistenState.then((fn) => fn());
      void unlistenPhase.then((fn) => fn());
      void unlistenTodayStats.then((fn) => fn());
      void unlistenBlockingError.then((fn) => fn());
    };
  }, [setFromBackend, setTodayStats, setBlockingError]);

  return useMemo(
    () => ({
      start: async () => {
        await invoke("start_timer");
      },
      pause: async () => {
        await invoke("pause_timer");
      },
      reset: async () => {
        await invoke("reset_timer");
      },
    }),
    [],
  );
}
