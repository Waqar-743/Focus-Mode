use std::sync::{Arc, Mutex};

use chrono::{Duration as ChronoDuration, Utc};
use rusqlite::{params, Connection, OptionalExtension};
use tauri::AppHandle;
use tauri::Manager;

use crate::models::{Settings, TodayStatistics};
use crate::timer::Phase;

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn new(app: &AppHandle) -> Result<Self, String> {
        let mut dir = app
            .path()
            .app_data_dir()
            .map_err(|e| format!("failed to resolve app data dir: {e}"))?;
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("failed to create app data dir {dir:?}: {e}"))?;
        dir.push("focus_timer.sqlite3");

        let conn = Connection::open(&dir)
            .map_err(|e| format!("failed to open database {dir:?}: {e}"))?;

        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.migrate()?;
        db.ensure_default_settings()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "db mutex poisoned".to_string())?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS settings (
                id INTEGER PRIMARY KEY CHECK (id = 1),
                work_minutes INTEGER NOT NULL,
                short_break_minutes INTEGER NOT NULL,
                long_break_minutes INTEGER NOT NULL,
                long_break_after INTEGER NOT NULL,
                daily_goal INTEGER NOT NULL
            );

            CREATE TABLE IF NOT EXISTS sessions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                date TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT NOT NULL,
                duration_minutes INTEGER NOT NULL,
                phase_type TEXT NOT NULL,
                completed INTEGER NOT NULL DEFAULT 1,
                notes TEXT
            );",
        )
        .map_err(|e| format!("failed to migrate database: {e}"))?;

        Ok(())
    }

    fn ensure_default_settings(&self) -> Result<(), String> {
        let existing = self.load_settings().ok();
        if existing.is_some() {
            return Ok(());
        }

        self.save_settings(&Settings::default())
    }

    pub fn load_settings(&self) -> Result<Settings, String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "db mutex poisoned".to_string())?;

        let row = conn
            .query_row(
                "SELECT work_minutes, short_break_minutes, long_break_minutes, long_break_after, daily_goal
                 FROM settings WHERE id = 1",
                [],
                |r| {
                    Ok(Settings {
                        work_minutes: r.get::<_, i64>(0)? as u32,
                        short_break_minutes: r.get::<_, i64>(1)? as u32,
                        long_break_minutes: r.get::<_, i64>(2)? as u32,
                        long_break_after: r.get::<_, i64>(3)? as u32,
                        daily_goal: r.get::<_, i64>(4)? as u32,
                    })
                },
            )
            .optional()
            .map_err(|e| format!("failed to read settings: {e}"))?;

        row.ok_or_else(|| "settings not initialized".to_string())
    }

    pub fn save_settings(&self, settings: &Settings) -> Result<(), String> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| "db mutex poisoned".to_string())?;

        conn.execute(
            "INSERT INTO settings (id, work_minutes, short_break_minutes, long_break_minutes, long_break_after, daily_goal)
             VALUES (1, ?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(id) DO UPDATE SET
               work_minutes = excluded.work_minutes,
               short_break_minutes = excluded.short_break_minutes,
               long_break_minutes = excluded.long_break_minutes,
               long_break_after = excluded.long_break_after,
               daily_goal = excluded.daily_goal",
            params![
                settings.work_minutes as i64,
                settings.short_break_minutes as i64,
                settings.long_break_minutes as i64,
                settings.long_break_after as i64,
                settings.daily_goal as i64
            ],
        )
        .map_err(|e| format!("failed to save settings: {e}"))?;

        Ok(())
    }

    pub fn insert_completed_phase(&self, phase: Phase, duration_seconds: u32) -> Result<(), String> {
        let now = Utc::now();
        let start = now - ChronoDuration::seconds(duration_seconds as i64);
        let date = now.date_naive().to_string();
        let phase_type = match phase {
            Phase::Work => "work",
            Phase::ShortBreak | Phase::LongBreak => "break",
        };

        let conn = self
            .conn
            .lock()
            .map_err(|_| "db mutex poisoned".to_string())?;

        conn.execute(
            "INSERT INTO sessions (date, start_time, end_time, duration_minutes, phase_type, completed, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, 1, NULL)",
            params![
                date,
                start.to_rfc3339(),
                now.to_rfc3339(),
                (duration_seconds / 60) as i64,
                phase_type
            ],
        )
        .map_err(|e| format!("failed to insert session: {e}"))?;

        Ok(())
    }

    pub fn get_today_statistics(&self) -> Result<TodayStatistics, String> {
        let settings = self.load_settings()?;
        let date = Utc::now().date_naive().to_string();

        let conn = self
            .conn
            .lock()
            .map_err(|_| "db mutex poisoned".to_string())?;

        let total_focus_minutes: u32 = conn
            .query_row(
                "SELECT COALESCE(SUM(duration_minutes), 0)
                 FROM sessions
                 WHERE date = ?1 AND phase_type = 'work' AND completed = 1",
                params![date.clone()],
                |r| Ok(r.get::<_, i64>(0)? as u32),
            )
            .map_err(|e| format!("failed to compute total focus minutes: {e}"))?;

        let work_sessions_completed: u32 = conn
            .query_row(
                "SELECT COALESCE(COUNT(*), 0)
                 FROM sessions
                 WHERE date = ?1 AND phase_type = 'work' AND completed = 1",
                params![date.clone()],
                |r| Ok(r.get::<_, i64>(0)? as u32),
            )
            .map_err(|e| format!("failed to compute sessions completed: {e}"))?;

        let breaks_taken: u32 = conn
            .query_row(
                "SELECT COALESCE(COUNT(*), 0)
                 FROM sessions
                 WHERE date = ?1 AND phase_type = 'break' AND completed = 1",
                params![date.clone()],
                |r| Ok(r.get::<_, i64>(0)? as u32),
            )
            .map_err(|e| format!("failed to compute breaks taken: {e}"))?;

        let longest_streak: u32 = conn
            .query_row(
                "SELECT COALESCE(MAX(cnt), 0) FROM (
                    SELECT date, COUNT(*) AS cnt
                    FROM sessions
                    WHERE phase_type = 'work' AND completed = 1
                    GROUP BY date
                )",
                [],
                |r| Ok(r.get::<_, i64>(0)? as u32),
            )
            .map_err(|e| format!("failed to compute longest streak: {e}"))?;

        Ok(TodayStatistics {
            date,
            total_focus_minutes,
            work_sessions_completed,
            breaks_taken,
            current_streak: work_sessions_completed,
            longest_streak,
            daily_goal: settings.daily_goal,
        })
    }
}
