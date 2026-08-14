//! Manual regression test for multi-window redraw starvation on Windows.
//!
//! Run with `cargo run --example issue_3648`. Each visible window should keep receiving redraws
//! while unfocused, resized, minimized/restored, and after focus is switched between windows.

use std::collections::HashMap;
use std::time::{Duration, Instant};

use winit::application::ApplicationHandler;
use winit::event::WindowEvent;
use winit::event_loop::{ActiveEventLoop, EventLoop};
use winit::window::{Window, WindowId};

const WINDOW_COUNT: usize = 3;
const REPORT_INTERVAL: Duration = Duration::from_secs(1);

#[derive(Default)]
struct RedrawTest {
    windows: HashMap<WindowId, Window>,
    redraws: HashMap<WindowId, u64>,
    last_report: Option<Instant>,
}

impl ApplicationHandler for RedrawTest {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if !self.windows.is_empty() {
            return;
        }

        for index in 0..WINDOW_COUNT {
            let window = event_loop
                .create_window(
                    Window::default_attributes()
                        .with_title(format!("winit #3648 redraw window {}", index + 1)),
                )
                .expect("failed to create test window");
            self.redraws.insert(window.id(), 0);
            self.windows.insert(window.id(), window);
        }

        self.last_report = Some(Instant::now());
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        window_id: WindowId,
        event: WindowEvent,
    ) {
        match event {
            WindowEvent::RedrawRequested => {
                *self.redraws.entry(window_id).or_default() += 1;
            },
            WindowEvent::CloseRequested => {
                self.windows.remove(&window_id);
                self.redraws.remove(&window_id);

                if self.windows.is_empty() {
                    event_loop.exit();
                }
            },
            _ => {},
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        for window in self.windows.values() {
            if window.is_visible() != Some(false) {
                window.request_redraw();
            }
        }

        let now = Instant::now();
        if self.last_report.is_some_and(|last| now.duration_since(last) >= REPORT_INTERVAL) {
            let mut counts: Vec<_> = self.redraws.iter().collect();
            counts.sort_unstable_by_key(|(id, _)| format!("{id:?}"));
            eprintln!("redraw totals: {counts:?}");
            self.last_report = Some(now);
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let event_loop = EventLoop::new()?;
    event_loop.run_app(&mut RedrawTest::default())?;
    Ok(())
}
