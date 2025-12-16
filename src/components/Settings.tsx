import { useMemo, useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useTimerStore } from "../store/timerStore";
import type { Settings as SettingsModel, TodayStatistics } from "../types/timer";

const DEFAULT_SETTINGS: SettingsModel = {
  workMinutes: 25,
  shortBreakMinutes: 5,
  longBreakMinutes: 15,
  longBreakAfter: 4,
  dailyGoal: 8,
};

function toInt(value: string): number {
  const n = Number(value);
  if (!Number.isFinite(n)) return 0;
  return Math.trunc(n);
}

export function Settings(props: { onClose: () => void }) {
  const current = useTimerStore((s) => s.settings);
  const setSettings = useTimerStore((s) => s.setSettings);
  const setTodayStats = useTimerStore((s) => s.setTodayStats);

  const [draft, setDraft] = useState<SettingsModel>(current ?? DEFAULT_SETTINGS);
  const [saving, setSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const canSave = useMemo(() => {
    return !saving;
  }, [saving]);

  const onSave = async () => {
    setSaving(true);
    setError(null);
    try {
      const saved = (await invoke("save_settings", { settings: draft })) as SettingsModel;
      setSettings(saved);

      try {
        const stats = (await invoke("get_today_statistics")) as TodayStatistics;
        setTodayStats(stats);
      } catch {
        // Ignore
      }

      props.onClose();
    } catch (e) {
      const msg = typeof e === "string" ? e : "Failed to save settings";
      setError(msg);
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="modal" role="dialog" aria-modal="true" aria-label="Settings">
      <div className="modal__backdrop" onClick={props.onClose} />
      <div className="modal__panel">
        <div className="modal__header">
          <h2 className="modal__title">Settings</h2>
          <button className="iconBtn" type="button" onClick={props.onClose} aria-label="Close">
            ✕
          </button>
        </div>

        <div className="form">
          <label className="field">
            <span className="field__label">Work Time (minutes)</span>
            <input
              className="field__input"
              type="number"
              min={1}
              max={60}
              value={draft.workMinutes}
              onChange={(e) =>
                setDraft((d) => ({ ...d, workMinutes: toInt(e.target.value) }))
              }
            />
          </label>

          <label className="field">
            <span className="field__label">Short Break (minutes)</span>
            <input
              className="field__input"
              type="number"
              min={1}
              max={30}
              value={draft.shortBreakMinutes}
              onChange={(e) =>
                setDraft((d) => ({ ...d, shortBreakMinutes: toInt(e.target.value) }))
              }
            />
          </label>

          <label className="field">
            <span className="field__label">Long Break (minutes)</span>
            <input
              className="field__input"
              type="number"
              min={1}
              max={30}
              value={draft.longBreakMinutes}
              onChange={(e) =>
                setDraft((d) => ({ ...d, longBreakMinutes: toInt(e.target.value) }))
              }
            />
          </label>

          <label className="field">
            <span className="field__label">Long Break After (sessions)</span>
            <input
              className="field__input"
              type="number"
              min={2}
              max={10}
              value={draft.longBreakAfter}
              onChange={(e) =>
                setDraft((d) => ({ ...d, longBreakAfter: toInt(e.target.value) }))
              }
            />
          </label>

          <label className="field">
            <span className="field__label">Daily Goal (sessions)</span>
            <input
              className="field__input"
              type="number"
              min={1}
              max={20}
              value={draft.dailyGoal}
              onChange={(e) => setDraft((d) => ({ ...d, dailyGoal: toInt(e.target.value) }))}
            />
          </label>
        </div>

        {error && <div className="form__error">{error}</div>}

        <div className="modal__actions">
          <button
            className="btn btn--ghost"
            type="button"
            onClick={() => setDraft(DEFAULT_SETTINGS)}
          >
            Reset to defaults
          </button>

          <div className="modal__actionsRight">
            <button className="btn btn--secondary" type="button" onClick={props.onClose}>
              Cancel
            </button>
            <button
              className="btn btn--primary"
              type="button"
              disabled={!canSave}
              onClick={() => void onSave()}
            >
              {saving ? "Saving…" : "Save"}
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
