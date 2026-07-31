// SPDX-License-Identifier: Apache-2.0
//! Wires xengui's platform-agnostic async executor into winit's event
//! loop, so a task's `Waker` - which may be invoked from a background
//! thread (a timer, an HTTP client's I/O driver, ...) - can wake a
//! blocked `ControlFlow::Wait` loop back up.

use winit::event_loop::EventLoopProxy;
use xengui::task::ExecutorWaker;

use crate::event::XenEvent;

pub struct WinitExecutorWaker(pub EventLoopProxy<XenEvent>);

impl ExecutorWaker for WinitExecutorWaker {
    fn wake(&self) {
        let _ = self.0.send_event(XenEvent::PollTasks);
    }
}
