// 事件系统 — 异步事件驱动架构
//
// 使用 crossterm EventStream 异步读取终端事件，通过 mpsc 通道传递给主循环。
// 事件循环运行在独立 tokio 任务中，与渲染分离。
//
// 关键：Windows 控制台对每个按键发送 Press+Release 成对事件，
// 在事件循环层做 120ms 去重，避免 Tab 跳两次、Enter 重复触发。

use std::time::{Duration, Instant};

use crossterm::event::{Event as CrosstermEvent, EventStream, KeyCode, KeyEventKind};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// 应用事件（封装 crossterm 事件 + 自定义事件）
#[derive(Debug, Clone)]
pub enum AppEvent {
    /// 定时刷新
    Tick,
    /// 键盘按键
    Key(KeyCode),
    /// 窗口大小变化
    Resize(u16, u16),
    /// 安装进度（包名, 百分比）
    InstallProgress(String, u8),
    /// 安装完成（包名, 是否成功）
    InstallComplete(String, bool),
    /// 退出
    Quit,
}

/// 启动事件循环（运行在独立 tokio 任务中）
///
/// 返回事件接收端，主循环从中消费事件。
pub fn start() -> (UnboundedSender<AppEvent>, UnboundedReceiver<AppEvent>) {
    let (tx, rx) = mpsc::unbounded_channel::<AppEvent>();

    // 内部 clone 一份 sender 给事件循环任务
    let loop_tx = tx.clone();
    tokio::spawn(async move {
        event_loop(loop_tx).await;
    });

    (tx, rx)
}

/// 事件循环主体
async fn event_loop(tx: UnboundedSender<AppEvent>) {
    use tokio_stream::StreamExt;

    let mut reader = EventStream::new();
    let mut tick_interval = tokio::time::interval(Duration::from_millis(100));

    // 去重状态：记录上一次按键的 code 和时间
    let mut last_key: Option<(KeyCode, Instant)> = None;

    loop {
        tokio::select! {
            // 定时 Tick（保证 UI 持续刷新）
            _ = tick_interval.tick() => {
                let _ = tx.send(AppEvent::Tick);
            }
            // 终端事件
            maybe_event = reader.next() => {
                match maybe_event {
                    Some(Ok(event)) => {
                        handle_crossterm_event(event, &tx, &mut last_key);
                    }
                    Some(Err(_)) => {
                        // 读取错误，发送退出
                        let _ = tx.send(AppEvent::Quit);
                        break;
                    }
                    None => {
                        // 流结束
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
