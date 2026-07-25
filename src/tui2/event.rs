// Event system — async event-driven architecture
//
// Uses crossterm EventStream for async terminal events via mpsc channel.
// Event loop runs in separate tokio task, decoupled from rendering.
//
// CRITICAL: Windows console sends Press+Release pairs for each keypress,
// 120ms dedup at event loop layer prevents Tab double-toggle, Enter double-fire.

use std::time::{Duration, Instant};

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode, KeyEventKind};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// App event (wraps crossterm + custom events)
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// Tick
    Tick,
    /// Key
    Key(KeyCode),
    /// Resize
    Resize(u16, u16),
    /// Install progress (package, %)
    InstallProgress(String, u8),
    /// Install complete (package, success)
    InstallComplete(String, bool),
    /// Quit
    Quit,
}

/// Start event loop (separate tokio task)
///
/// Returns event receiver for main loop consumption.
pub fn start() -> (UnboundedSender<AppEvent>, UnboundedReceiver<AppEvent>) {
    let (tx, rx) = mpsc::unbounded_channel::<AppEvent>();

    // internal clone of sender for event loop task
    let loop_tx = tx.clone();
    tokio::spawn(async move {
        event_loop(loop_tx).await;
    });

    (tx, rx)
}

/// Event loop body
async fn event_loop(tx: UnboundedSender<AppEvent>) {
    use tokio_stream::StreamExt;

    let mut reader = EventStream::new();
    let mut tick_interval = tokio::time::interval(Duration::from_millis(100));

    // Dedup state: track last key code and timestamp
    let mut last_key: Option<(KeyCode, Instant)> = None;

    loop {
        tokio::select! {
            // Tick timer (keeps UI refreshed)
            _ = tick_interval.tick() => {
                let _ = tx.send(AppEvent::Tick);
            }
            // Terminal event
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        handle_crossterm_event(event, &tx, &mut last_key);
                    }
                    Some(Err(_)) => {
                        // 读取错误，发送Quit
                        let _ = tx.send(AppEvent::Quit);
                        break;
                    }
                    None => {
                        // Stream ended
                        let _ = tx.send(AppEvent::Quit);
                        break;
                    }
                }
            }
        }
    }
}

/// handle single crossterm event
fn handle_crossterm_event(
    event: CrosstermEvent,
    tx: &UnboundedSender<AppEvent>,
    last_key: &mut Option<(KeyCode, Instant)>,
) {
    match event {
        CrosstermEvent::Key(key) => {
            // Windows console sends Press+Release pairs。
            // only handle Press（keep Repeat for long press），discard Release。
            // some terminals send Release first，因此对 Release 也做去重而非直接丢弃。
            match key.kind {
                KeyEventKind::Press | KeyEventKind::Repeat => {}
                KeyEventKind::Release => {
                    // 标记时间，让后续可能的重复 Press 被去重
                    record_key(last_key, key.code);
                    return;
                }
            }

            // 去重：同一 key 在 120ms 内重复到达则丢弃
            if is_repeated(last_key, key.code) {
                return;
            }

            if key.code == KeyCode::Char('q') || key.code == KeyCode::Esc {
                let _ = tx.send(AppEvent::Quit);
            } else {
                let _ = tx.send(AppEvent::Key(key.code));
            }
        }
        CrosstermEvent::Resize(w, h) => {
            let _ = tx.send(AppEvent::Resize(w, h));
        }
        _ => {}
    }
}

/// 判断是否为 120ms 内的重复按键
fn is_repeated(last_key: &mut Option<(KeyCode, Instant)>, code: KeyCode) -> bool {
    let now = Instant::now();
    let is_dup = last_key
        .map(|(lc, t)| lc == code && now.duration_since(t) < Duration::from_millis(120))
        .unwrap_or(false);
    *last_key = Some((code, now));
    is_dup
}

/// record key time without duplicate check
fn record_key(last_key: &mut Option<(KeyCode, Instant)>, code: KeyCode) {
    *last_key = Some((code, Instant::now()));
}
