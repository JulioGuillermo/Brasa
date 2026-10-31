//! Copies windows into memory through Wayland's standard capture protocols, which Hyprland and sway
//! both offer: the window list names each window by an identifier, a capture source is made from it,
//! and a capture session copies the window into a shared memory buffer.

use std::collections::HashMap;
use std::fs::File;
use std::os::fd::{AsFd, OwnedFd};
use std::os::unix::fs::FileExt;
use std::time::{Duration, Instant};

use rustix::event::{PollFd, PollFlags, Timespec, poll};
use rustix::fs::{MemfdFlags, memfd_create};
use wayland_client::globals::{GlobalListContents, registry_queue_init};
use wayland_client::protocol::{wl_buffer, wl_registry, wl_shm, wl_shm_pool};
use wayland_client::{Connection, Dispatch, EventQueue, Proxy, QueueHandle, WEnum, event_created_child};
use wayland_protocols::ext::foreign_toplevel_list::v1::client::{
    ext_foreign_toplevel_handle_v1 as toplevel, ext_foreign_toplevel_list_v1 as toplevel_list,
};
use wayland_protocols::ext::image_capture_source::v1::client::{
    ext_foreign_toplevel_image_capture_source_manager_v1 as source_manager, ext_image_capture_source_v1 as source,
};
use wayland_protocols::ext::image_copy_capture::v1::client::{
    ext_image_copy_capture_frame_v1 as frame, ext_image_copy_capture_manager_v1 as capture_manager,
    ext_image_copy_capture_session_v1 as session,
};

use super::shrink::BYTES_PER_PIXEL;

const TIME_ALLOWED: Duration = Duration::from_millis(1500);
const USABLE_FORMATS: [wl_shm::Format; 4] =
    [wl_shm::Format::Argb8888, wl_shm::Format::Xrgb8888, wl_shm::Format::Abgr8888, wl_shm::Format::Xbgr8888];

/// A window copied at its full size: rows `stride` bytes apart, 4 bytes a pixel.
pub struct Copied {
    pub capture_id: String,
    pub width: usize,
    pub height: usize,
    pub stride: usize,
    pub format: wl_shm::Format,
    pub pixels: Vec<u8>,
}

/// Blocks until every window is copied, or failed, or the time allowed is over. A window the
/// compositor does not know, or will not copy, is left out.
pub fn copy_windows(capture_ids: &[String]) -> Vec<Copied> {
    let Ok(connection) = Connection::connect_to_env() else { return Vec::new() };
    let Ok((globals, mut queue)) = registry_queue_init::<State>(&connection) else { return Vec::new() };
    let handle = queue.handle();
    let bound = (
        globals.bind::<wl_shm::WlShm, _, _>(&handle, 1..=1, ()),
        globals.bind::<toplevel_list::ExtForeignToplevelListV1, _, _>(&handle, 1..=1, ()),
        globals.bind::<source_manager::ExtForeignToplevelImageCaptureSourceManagerV1, _, _>(&handle, 1..=1, ()),
        globals.bind::<capture_manager::ExtImageCopyCaptureManagerV1, _, _>(&handle, 1..=1, ()),
    );
    let (Ok(shm), Ok(_list), Ok(sources), Ok(captures)) = bound else { return Vec::new() };

    let mut state = State::default();
    if queue.roundtrip(&mut state).is_err() {
        return Vec::new();
    }
    for capture_id in capture_ids {
        let Some(window) = state.toplevels.get(capture_id) else { continue };
        let source = sources.create_source(window, &handle, ());
        let index = state.captures.len();
        let session = captures.create_session(&source, capture_manager::Options::empty(), &handle, index);
        state.captures.push(Capture::new(capture_id, session));
    }
    if state.captures.is_empty() || queue.roundtrip(&mut state).is_err() {
        return Vec::new();
    }
    for (index, capture) in state.captures.iter_mut().enumerate() {
        capture.start(index, &shm, &handle);
    }
    wait_for_frames(&connection, &mut queue, &mut state);
    state.captures.into_iter().filter_map(Capture::into_copied).collect()
}

fn wait_for_frames(connection: &Connection, queue: &mut EventQueue<State>, state: &mut State) {
    let deadline = Instant::now() + TIME_ALLOWED;
    while state.captures.iter().any(Capture::is_waiting) {
        let Some(left) = deadline.checked_duration_since(Instant::now()) else { return };
        if connection.flush().is_err() {
            return;
        }
        if let Some(guard) = queue.prepare_read() {
            let socket = guard.connection_fd();
            let mut readable = [PollFd::new(&socket, PollFlags::IN)];
            let timeout = Timespec::try_from(left).unwrap_or(Timespec { tv_sec: 1, tv_nsec: 0 });
            let ready = poll(&mut readable, Some(&timeout)).unwrap_or(0) > 0;
            if ready && guard.read().is_err() {
                return;
            }
        }
        if queue.dispatch_pending(state).is_err() {
            return;
        }
    }
}

#[derive(Default)]
struct State {
    toplevels: HashMap<String, toplevel::ExtForeignToplevelHandleV1>,
    captures: Vec<Capture>,
}

#[derive(PartialEq, Eq)]
enum Progress {
    /// The session is telling which buffers it takes.
    Agreeing,
    Copying,
    Copied,
    Failed,
}

struct Capture {
    capture_id: String,
    size: Option<(usize, usize)>,
    formats: Vec<wl_shm::Format>,
    session: session::ExtImageCopyCaptureSessionV1,
    memory: Option<OwnedFd>,
    format: Option<wl_shm::Format>,
    progress: Progress,
}

impl Capture {
    fn new(capture_id: &str, session: session::ExtImageCopyCaptureSessionV1) -> Capture {
        Capture {
            capture_id: capture_id.to_string(),
            size: None,
            formats: Vec::new(),
            session,
            memory: None,
            format: None,
            progress: Progress::Agreeing,
        }
    }

    fn is_waiting(&self) -> bool {
        self.progress == Progress::Copying
    }

    /// A shared memory buffer of the size and format the session asked for, and the copy into it.
    fn start(&mut self, index: usize, shm: &wl_shm::WlShm, handle: &QueueHandle<State>) {
        if self.progress != Progress::Agreeing {
            return;
        }
        self.progress = Progress::Failed;
        let Some((width, height)) = self.size else { return };
        let Some(format) = USABLE_FORMATS.into_iter().find(|format| self.formats.contains(format)) else { return };
        let stride = width * BYTES_PER_PIXEL;
        let Ok(memory) = memfd_create("brasa-window-shot", MemfdFlags::CLOEXEC) else { return };
        if rustix::fs::ftruncate(&memory, (stride * height) as u64).is_err() {
            return;
        }
        let pool = shm.create_pool(memory.as_fd(), (stride * height) as i32, handle, ());
        let buffer = pool.create_buffer(0, width as i32, height as i32, stride as i32, format, handle, ());
        pool.destroy();
        let copy = self.session.create_frame(handle, index);
        copy.attach_buffer(&buffer);
        copy.damage_buffer(0, 0, width as i32, height as i32);
        copy.capture();
        self.memory = Some(memory);
        self.format = Some(format);
        self.progress = Progress::Copying;
    }

    fn into_copied(self) -> Option<Copied> {
        if self.progress != Progress::Copied {
            return None;
        }
        let (width, height) = self.size?;
        let stride = width * BYTES_PER_PIXEL;
        let mut pixels = vec![0u8; stride * height];
        File::from(self.memory?).read_exact_at(&mut pixels, 0).ok()?;
        Some(Copied { capture_id: self.capture_id, width, height, stride, format: self.format?, pixels })
    }
}

impl Dispatch<session::ExtImageCopyCaptureSessionV1, usize> for State {
    fn event(
        state: &mut Self,
        _: &session::ExtImageCopyCaptureSessionV1,
        event: session::Event,
        index: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let Some(capture) = state.captures.get_mut(*index) else { return };
        match event {
            session::Event::BufferSize { width, height } => capture.size = Some((width as usize, height as usize)),
            session::Event::ShmFormat { format: WEnum::Value(format) } => capture.formats.push(format),
            session::Event::Stopped => capture.progress = Progress::Failed,
            _ => {}
        }
    }
}

impl Dispatch<frame::ExtImageCopyCaptureFrameV1, usize> for State {
    fn event(
        state: &mut Self,
        copy: &frame::ExtImageCopyCaptureFrameV1,
        event: frame::Event,
        index: &usize,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        let progress = match event {
            frame::Event::Ready => Progress::Copied,
            frame::Event::Failed { .. } => Progress::Failed,
            _ => return,
        };
        copy.destroy();
        if let Some(capture) = state.captures.get_mut(*index) {
            capture.progress = progress;
        }
    }
}

impl Dispatch<toplevel_list::ExtForeignToplevelListV1, ()> for State {
    fn event(
        _: &mut Self,
        _: &toplevel_list::ExtForeignToplevelListV1,
        _: toplevel_list::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }

    event_created_child!(State, toplevel_list::ExtForeignToplevelListV1, [
        toplevel_list::EVT_TOPLEVEL_OPCODE => (toplevel::ExtForeignToplevelHandleV1, ()),
    ]);
}

impl Dispatch<toplevel::ExtForeignToplevelHandleV1, ()> for State {
    fn event(
        state: &mut Self,
        window: &toplevel::ExtForeignToplevelHandleV1,
        event: toplevel::Event,
        _: &(),
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
        if let toplevel::Event::Identifier { identifier } = event {
            state.toplevels.insert(identifier, window.clone());
        }
    }
}

macro_rules! ignore_events {
    ($($interface:ty),*) => {$(
        impl Dispatch<$interface, ()> for State {
            fn event(_: &mut Self, _: &$interface, _: <$interface as Proxy>::Event, _: &(), _: &Connection, _: &QueueHandle<Self>) {}
        }
    )*};
}

ignore_events!(
    wl_shm::WlShm,
    wl_shm_pool::WlShmPool,
    wl_buffer::WlBuffer,
    source_manager::ExtForeignToplevelImageCaptureSourceManagerV1,
    source::ExtImageCaptureSourceV1,
    capture_manager::ExtImageCopyCaptureManagerV1
);

impl Dispatch<wl_registry::WlRegistry, GlobalListContents> for State {
    fn event(
        _: &mut Self,
        _: &wl_registry::WlRegistry,
        _: wl_registry::Event,
        _: &GlobalListContents,
        _: &Connection,
        _: &QueueHandle<Self>,
    ) {
    }
}

