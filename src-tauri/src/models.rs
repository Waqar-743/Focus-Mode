use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub work_minutes: u32,
    pub short_break_minutes: u32,
    pub long_break_minutes: u32,
    pub long_break_after: u32,
    pub daily_goal: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            work_minutes: 25,
            short_break_minutes: 5,
            long_break_minutes: 15,
            long_break_after: 4,
            daily_goal: 8,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TodayStatistics {
    pub date: String,
    pub total_focus_minutes: u32,
    pub work_sessions_completed: u32,
    pub breaks_taken: u32,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub daily_goal: u32,
}
