use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Work,
    ShortBreak,
    LongBreak,
}

#[derive(Debug, Clone, Copy)]
pub struct TimerConfig {
    pub work_seconds: u32,
    pub short_break_seconds: u32,
    pub long_break_seconds: u32,
    pub long_break_after: u32,
    pub daily_goal: u32,
}

impl Default for TimerConfig {
    fn default() -> Self {
        Self {
            work_seconds: 25 * 60,
            short_break_seconds: 5 * 60,
            long_break_seconds: 15 * 60,
            long_break_after: 4,
            daily_goal: 8,
        }
    }
}

impl TimerConfig {
    pub fn seconds_for(self, phase: Phase) -> u32 {
        match phase {
            Phase::Work => self.work_seconds,
            Phase::ShortBreak => self.short_break_seconds,
            Phase::LongBreak => self.long_break_seconds,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimerState {
    pub phase: Phase,
    pub total_seconds: u32,
    pub remaining_seconds: u32,
    pub is_running: bool,
    pub completed_work_sessions: u32,
    pub long_break_after: u32,
    pub daily_goal: u32,
}

impl TimerState {
    pub fn new(cfg: TimerConfig) -> Self {
        let total = cfg.seconds_for(Phase::Work);
        Self {
            phase: Phase::Work,
            total_seconds: total,
            remaining_seconds: total,
            is_running: false,
            completed_work_sessions: 0,
            long_break_after: cfg.long_break_after,
            daily_goal: cfg.daily_goal,
        }
    }

    pub fn reset_current_phase(&mut self) {
        self.remaining_seconds = self.total_seconds;
        self.is_running = false;
    }

    pub fn apply_phase(&mut self, phase: Phase, cfg: TimerConfig) {
        self.phase = phase;
        self.total_seconds = cfg.seconds_for(phase);
        self.remaining_seconds = self.total_seconds;
    }

    pub fn tick(&mut self, cfg: TimerConfig) -> Option<PhaseCompleteEvent> {
        if !self.is_running {
            return None;
        }

        if self.remaining_seconds > 0 {
            self.remaining_seconds -= 1;
        }

        if self.remaining_seconds != 0 {
            return None;
        }

        let from = self.phase;
        let completed_seconds = self.total_seconds;
        let to = match from {
            Phase::Work => {
                self.completed_work_sessions = self.completed_work_sessions.saturating_add(1);
                if cfg.long_break_after > 0
                    && self.completed_work_sessions % cfg.long_break_after == 0
                {
                    Phase::LongBreak
                } else {
                    Phase::ShortBreak
                }
            }
            Phase::ShortBreak | Phase::LongBreak => Phase::Work,
        };

        self.apply_phase(to, cfg);

        Some(PhaseCompleteEvent {
            from,
            to,
            completed_seconds,
        })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhaseCompleteEvent {
    pub from: Phase,
    pub to: Phase,
    pub completed_seconds: u32,
}
