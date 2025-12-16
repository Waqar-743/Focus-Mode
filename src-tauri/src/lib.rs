mod database;
mod focus_blocker;
mod models;
mod timer;

use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager, State, WindowEvent};

use database::Database;
use focus_blocker::{emit_blocking_error, FocusBlocker};
use models::{Settings, TodayStatistics};
use timer::{Phase, PhaseCompleteEvent, TimerConfig, TimerState};

#[derive(Clone)]
struct TimerManager {
    cfg: Arc<Mutex<TimerConfig>>,
    state: Arc<Mutex<TimerState>>,
    worker_running: Arc<AtomicBool>,
}

impl TimerManager {
    fn new(cfg: TimerConfig) -> Self {
        Self {
            cfg: Arc::new(Mutex::new(cfg)),
            state: Arc::new(Mutex::new(TimerState::new(cfg))),
            worker_running: Arc::new(AtomicBool::new(false)),
        }
    }
}

fn emit_timer_state(app: &AppHandle, state: &TimerState) {
    let _ = app.emit("timer-state", state);
}

fn emit_phase_complete(app: &AppHandle, evt: &PhaseCompleteEvent) {
    let _ = app.emit("phase-complete", evt);
}

fn emit_today_statistics(app: &AppHandle, stats: &TodayStatistics) {
    let _ = app.emit("today-statistics", stats);
}

fn timer_config_from_settings(settings: &Settings) -> TimerConfig {
    TimerConfig {
        work_seconds: settings.work_minutes.saturating_mul(60),
        short_break_seconds: settings.short_break_minutes.saturating_mul(60),
        long_break_seconds: settings.long_break_minutes.saturating_mul(60),
        long_break_after: settings.long_break_after,
        daily_goal: settings.daily_goal,
    }
}

fn spawn_timer_worker(app: AppHandle, manager: TimerManager, db: Database, blocker: FocusBlocker) {
    thread::spawn(move || {
        loop {
            let (snapshot, phase_complete, running) = {
                let cfg = manager
                    .cfg
                    .lock()
                    .expect("timer cfg mutex poisoned")
                    .clone();
                let mut state = manager
                    .state
                    .lock()
                    .expect("timer state mutex poisoned");

                if !state.is_running {
                    (state.clone(), None, false)
                } else {
                    let evt = state.tick(cfg);
                    (state.clone(), evt, true)
                }
            };

            if !running {
                break;
            }

            if let Some(evt) = phase_complete {
                if evt.to == Phase::Work {
                    if let Err(e) = blocker.enable() {
                        emit_blocking_error(
                            &app,
                            format!(
                                "Focus-mode blocking couldn't be enabled (try running the app as Administrator): {e}"
                            ),
                        );
                    }
                } else if let Err(e) = blocker.disable() {
                    emit_blocking_error(&app, format!("Failed to disable focus-mode blocking: {e}"));
                }

                let _ = db.insert_completed_phase(evt.from, evt.completed_seconds);
                if let Ok(stats) = db.get_today_statistics() {
                    emit_today_statistics(&app, &stats);
                }
                emit_phase_complete(&app, &evt);
            }
            emit_timer_state(&app, &snapshot);

            thread::sleep(Duration::from_secs(1));
        }

        manager.worker_running.store(false, Ordering::SeqCst);
    });
}

#[tauri::command]
fn initialize_timer(app: AppHandle, manager: State<'_, TimerManager>) -> Result<TimerState, String> {
    if let Some(blocker) = app.try_state::<FocusBlocker>() {
        let _ = blocker.inner().disable();
    }
    let snapshot = get_timer_state(manager)?;
    emit_timer_state(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn get_timer_state(manager: State<'_, TimerManager>) -> Result<TimerState, String> {
    let state = manager
        .state
        .lock()
        .map_err(|_| "timer state mutex poisoned".to_string())?;
    Ok(state.clone())
}

#[tauri::command]
fn start_timer(app: AppHandle, manager: State<'_, TimerManager>) -> Result<(), String> {
    {
        let mut state = manager
            .state
            .lock()
            .map_err(|_| "timer state mutex poisoned".to_string())?;
        state.is_running = true;
    }

    let snapshot = {
        manager
            .state
            .lock()
            .map_err(|_| "timer state mutex poisoned".to_string())?
            .clone()
    };
    emit_timer_state(&app, &snapshot);

    if let Some(blocker) = app.try_state::<FocusBlocker>() {
        if snapshot.phase == Phase::Work {
            if let Err(e) = blocker.inner().enable() {
                emit_blocking_error(
                    &app,
                    format!(
                        "Focus-mode blocking couldn't be enabled (try running the app as Administrator): {e}"
                    ),
                );
            }
        } else {
            let _ = blocker.inner().disable();
        }
    }

    if manager
        .worker_running
        .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
        .is_ok()
    {
        let db = app
            .try_state::<Database>()
            .ok_or_else(|| "database not initialized".to_string())?
            .inner()
            .clone();
        let blocker = app
            .try_state::<FocusBlocker>()
            .ok_or_else(|| "focus blocker not initialized".to_string())?
            .inner()
            .clone();
        spawn_timer_worker(app, manager.inner().clone(), db, blocker);
    }

    Ok(())
}

#[tauri::command]
fn get_settings(db: State<'_, Database>) -> Result<Settings, String> {
    db.load_settings()
}

fn validate_settings(settings: &Settings) -> Result<(), String> {
    fn in_range(v: u32, min: u32, max: u32, field: &str) -> Result<(), String> {
        if (min..=max).contains(&v) {
            Ok(())
        } else {
            Err(format!("{field} must be between {min} and {max}"))
        }
    }

    in_range(settings.work_minutes, 1, 60, "workMinutes")?;
    in_range(settings.short_break_minutes, 1, 30, "shortBreakMinutes")?;
    in_range(settings.long_break_minutes, 1, 30, "longBreakMinutes")?;
    in_range(settings.long_break_after, 2, 10, "longBreakAfter")?;
    in_range(settings.daily_goal, 1, 20, "dailyGoal")?;
    Ok(())
}

#[tauri::command]
fn save_settings(
    app: AppHandle,
    db: State<'_, Database>,
    manager: State<'_, TimerManager>,
    settings: Settings,
) -> Result<Settings, String> {
    validate_settings(&settings)?;
    db.save_settings(&settings)?;

    let new_cfg = timer_config_from_settings(&settings);
    {
        let mut cfg = manager
            .cfg
            .lock()
            .map_err(|_| "timer cfg mutex poisoned".to_string())?;
        *cfg = new_cfg;
    }

    {
        let mut state = manager
            .state
            .lock()
            .map_err(|_| "timer state mutex poisoned".to_string())?;

        state.long_break_after = new_cfg.long_break_after;
        state.daily_goal = new_cfg.daily_goal;

        if !state.is_running {
            let current_phase = state.phase;
            state.apply_phase(current_phase, new_cfg);
        }
    }

    if let Ok(snapshot) = get_timer_state(manager) {
        emit_timer_state(&app, &snapshot);
    }
    Ok(settings)
}

#[tauri::command]
fn get_today_statistics(db: State<'_, Database>) -> Result<TodayStatistics, String> {
    db.get_today_statistics()
}

#[tauri::command]
fn pause_timer(app: AppHandle, manager: State<'_, TimerManager>) -> Result<TimerState, String> {
    {
        let mut state = manager
            .state
            .lock()
            .map_err(|_| "timer state mutex poisoned".to_string())?;
        state.is_running = false;
    }

    if let Some(blocker) = app.try_state::<FocusBlocker>() {
        let _ = blocker.inner().disable();
    }

    let snapshot = get_timer_state(manager)?;
    emit_timer_state(&app, &snapshot);
    Ok(snapshot)
}

#[tauri::command]
fn reset_timer(app: AppHandle, manager: State<'_, TimerManager>) -> Result<TimerState, String> {
    {
        let mut state = manager
            .state
            .lock()
            .map_err(|_| "timer state mutex poisoned".to_string())?;
        state.reset_current_phase();
    }

    if let Some(blocker) = app.try_state::<FocusBlocker>() {
        let _ = blocker.inner().disable();
    }

    let snapshot = get_timer_state(manager)?;
    emit_timer_state(&app, &snapshot);
    Ok(snapshot)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let db = Database::new(&app.handle())?;
            let settings = db.load_settings().unwrap_or_default();
            let cfg = timer_config_from_settings(&settings);
            let blocker = FocusBlocker::default();
            blocker.ensure_disabled();
            app.manage(db);
            app.manage(TimerManager::new(cfg));
            app.manage(blocker);
            Ok(())
        })
        .on_window_event(|window, event| {
            if matches!(event, WindowEvent::CloseRequested { .. }) {
                if let Some(blocker) = window.app_handle().try_state::<FocusBlocker>() {
                    let _ = blocker.inner().disable();
                }
            }
        })
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            initialize_timer,
            get_timer_state,
            start_timer,
            pause_timer,
            reset_timer,
            get_settings,
            save_settings,
            get_today_statistics
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
