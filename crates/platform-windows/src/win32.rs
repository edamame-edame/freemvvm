use crate::input::{
    PointerDevice, map_key_message, map_pointer_button, map_utf16_char, modifier_mask,
    signed_client_pixels,
};
use crate::peer::NativeWindowBackend;
use crate::renderer::direct2d::HwndRenderer;
use freemvvm_core::abi_support::{
    DeviceScale, FrameTicket, PlatformOperationError, PlatformWindowClientAreaSink,
    PlatformWindowCloseRequestSink, PlatformWindowInputEvent, PlatformWindowInputSink,
    ResolvedTextSnapshot, SceneSnapshot, TextRequest, TextServiceError, WindowClientArea,
    WindowOpenInfo,
};
use std::ffi::c_void;
use std::mem::zeroed;
use std::ptr::{null, null_mut};
use std::sync::Arc;
use std::sync::mpsc::{self, Receiver, Sender, SyncSender, TryRecvError};
use std::thread::{self, JoinHandle};
use windows_sys::Win32::Foundation::POINT;
use windows_sys::Win32::Foundation::{
    CloseHandle, GetLastError, HWND, LPARAM, LRESULT, WAIT_FAILED, WAIT_OBJECT_0, WPARAM,
};
use windows_sys::Win32::Graphics::Gdi::{
    BeginPaint, EndPaint, InvalidateRect, PAINTSTRUCT, ScreenToClient, UpdateWindow,
};
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;
use windows_sys::Win32::System::Threading::{CreateEventW, INFINITE, ResetEvent, SetEvent};
use windows_sys::Win32::UI::HiDpi::GetDpiForWindow;
use windows_sys::Win32::UI::Input::KeyboardAndMouse::{
    GetKeyState, VK_CONTROL, VK_LWIN, VK_MENU, VK_RWIN, VK_SHIFT,
};
use windows_sys::Win32::UI::Input::Pointer::{GetPointerInfo, GetPointerType, POINTER_INFO};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    CREATESTRUCTW, CS_HREDRAW, CS_VREDRAW, CW_USEDEFAULT, CreateWindowExW, DefWindowProcW,
    DestroyWindow, DispatchMessageW, GWLP_USERDATA, GetClientRect, GetWindowLongPtrW, MSG,
    MsgWaitForMultipleObjects, PM_NOREMOVE, PM_REMOVE, POINTER_MESSAGE_FLAG_FIRSTBUTTON,
    POINTER_MESSAGE_FLAG_PRIMARY, PT_PEN, PT_TOUCH, PeekMessageW, RegisterClassW, SW_SHOW,
    SetWindowLongPtrW, ShowWindow, TranslateMessage, WA_INACTIVE, WM_ACTIVATE, WM_CHAR, WM_CLOSE,
    WM_DEADCHAR, WM_KEYDOWN, WM_KEYUP, WM_KILLFOCUS, WM_LBUTTONDOWN, WM_LBUTTONUP, WM_NCCREATE,
    WM_NCDESTROY, WM_PAINT, WM_POINTERDOWN, WM_POINTERUP, WM_SETFOCUS, WM_SIZE, WM_SYSCHAR,
    WM_SYSDEADCHAR, WM_SYSKEYDOWN, WM_SYSKEYUP, WNDCLASSW, WS_OVERLAPPEDWINDOW,
};

const CLASS_NAME: [u16; 18] = [
    b'F' as u16,
    b'r' as u16,
    b'e' as u16,
    b'e' as u16,
    b'M' as u16,
    b'v' as u16,
    b'v' as u16,
    b'm' as u16,
    b'W' as u16,
    b'i' as u16,
    b'n' as u16,
    b'd' as u16,
    b'o' as u16,
    b'w' as u16,
    b'P' as u16,
    b'e' as u16,
    b'e' as u16,
    b'r' as u16,
];
const CLASS_NAME_NUL: [u16; 19] = {
    let mut name = [0; 19];
    let mut index = 0;
    while index < CLASS_NAME.len() {
        name[index] = CLASS_NAME[index];
        index += 1;
    }
    name
};
const WINDOW_TITLE: [u16; 1] = [0];
enum Command {
    Open {
        generation: u64,
        sink: Option<Arc<dyn PlatformWindowClientAreaSink>>,
        close_request_sink: Option<Arc<dyn PlatformWindowCloseRequestSink>>,
        input_sink: Option<Arc<dyn PlatformWindowInputSink>>,
        response: SyncSender<Result<WindowOpenInfo, PlatformOperationError>>,
    },
    Close {
        response: SyncSender<Result<(), PlatformOperationError>>,
    },
    SubmitFrame {
        scene: Arc<SceneSnapshot>,
        response: SyncSender<Result<(), PlatformOperationError>>,
    },
    ResolveText {
        request: TextRequest,
        response: SyncSender<Result<ResolvedTextSnapshot, TextServiceError>>,
    },
    Shutdown,
}

struct Worker {
    sender: Sender<Command>,
    wake_event: usize,
    thread: Option<JoinHandle<()>>,
}

impl Worker {
    fn start() -> Result<Self, PlatformOperationError> {
        // SAFETY: a new unnamed event is created with no inherited handle or access attributes.
        let wake_event = unsafe { CreateEventW(null(), 1, 0, null()) };
        if wake_event.is_null() {
            return Err(PlatformOperationError);
        }

        let (sender, receiver) = mpsc::channel();
        let event_value = wake_event as usize;
        let (ready_sender, ready_receiver) = mpsc::sync_channel(1);
        let thread = match thread::Builder::new()
            .name("freemvvm-win32-window".to_owned())
            .spawn(move || worker_main(receiver, ready_sender, event_value))
        {
            Ok(thread) => thread,
            Err(_) => {
                // SAFETY: this function created the handle and no worker owns it on spawn failure.
                unsafe { CloseHandle(wake_event) };
                return Err(PlatformOperationError);
            }
        };

        match ready_receiver.recv() {
            Ok(()) => Ok(Self {
                sender,
                wake_event: event_value,
                thread: Some(thread),
            }),
            Err(_) => {
                let _ = thread.join();
                Err(PlatformOperationError)
            }
        }
    }

    fn send(&self, command: Command) -> Result<(), PlatformOperationError> {
        self.sender
            .send(command)
            .map_err(|_| PlatformOperationError)?;
        // SAFETY: the event remains owned by the worker until it exits; Worker joins before drop.
        if unsafe { SetEvent(self.wake_event as _) } == 0 {
            return Err(PlatformOperationError);
        }
        Ok(())
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        if let Some(thread) = self.thread.take() {
            let _ = self.sender.send(Command::Shutdown);
            // SAFETY: the worker owns and closes this handle only after it processes Shutdown.
            let _ = unsafe { SetEvent(self.wake_event as _) };
            let _ = thread.join();
        }
    }
}

pub(super) struct Backend {
    worker: Option<Worker>,
}

trait WorkerTextResolver {
    fn resolve_text(
        &mut self,
        request: &TextRequest,
    ) -> Result<ResolvedTextSnapshot, TextServiceError>;
}

impl WorkerTextResolver for HwndRenderer {
    fn resolve_text(
        &mut self,
        request: &TextRequest,
    ) -> Result<ResolvedTextSnapshot, TextServiceError> {
        HwndRenderer::resolve_text(self, request)
    }
}

fn resolve_text_command(
    renderer: Option<&mut dyn WorkerTextResolver>,
    request: &TextRequest,
) -> Result<ResolvedTextSnapshot, TextServiceError> {
    renderer
        .ok_or(TextServiceError::Unavailable)?
        .resolve_text(request)
}

impl Backend {
    pub(super) fn new() -> Self {
        Self { worker: None }
    }

    fn ensure_worker(&mut self) -> Result<&Worker, PlatformOperationError> {
        if self.worker.is_none() {
            self.worker = Some(Worker::start()?);
        }
        self.worker.as_ref().ok_or(PlatformOperationError)
    }
}

impl NativeWindowBackend for Backend {
    fn open(
        &mut self,
        generation: u64,
        sink: Option<Arc<dyn PlatformWindowClientAreaSink>>,
        close_request_sink: Option<Arc<dyn PlatformWindowCloseRequestSink>>,
        input_sink: Option<Arc<dyn PlatformWindowInputSink>>,
    ) -> Result<WindowOpenInfo, PlatformOperationError> {
        let worker = self.ensure_worker()?;
        let (response, result) = mpsc::sync_channel(1);
        worker.send(Command::Open {
            generation,
            sink,
            close_request_sink,
            input_sink,
            response,
        })?;
        result.recv().map_err(|_| PlatformOperationError)?
    }

    fn close(&mut self) -> Result<(), PlatformOperationError> {
        let Some(worker) = &self.worker else {
            return Ok(());
        };
        let (response, result) = mpsc::sync_channel(1);
        worker.send(Command::Close { response })?;
        result.recv().map_err(|_| PlatformOperationError)?
    }

    fn submit_frame(
        &mut self,
        _ticket: FrameTicket,
        scene: Arc<SceneSnapshot>,
    ) -> Result<(), PlatformOperationError> {
        let worker = self.worker.as_ref().ok_or(PlatformOperationError)?;
        let (response, result) = mpsc::sync_channel(1);
        worker.send(Command::SubmitFrame { scene, response })?;
        result.recv().map_err(|_| PlatformOperationError)?
    }

    fn resolve_text(
        &mut self,
        request: &TextRequest,
    ) -> Result<ResolvedTextSnapshot, TextServiceError> {
        let worker = self.worker.as_ref().ok_or(TextServiceError::Unavailable)?;
        let (response, result) = mpsc::sync_channel(1);
        worker
            .send(Command::ResolveText {
                request: request.clone(),
                response,
            })
            .map_err(|_| TextServiceError::Unavailable)?;
        result.recv().map_err(|_| TextServiceError::Unavailable)?
    }
}

struct WindowState {
    generation: u64,
    sink: Option<Arc<dyn PlatformWindowClientAreaSink>>,
    close_request_sink: Option<Arc<dyn PlatformWindowCloseRequestSink>>,
    input_sink: Option<Arc<dyn PlatformWindowInputSink>>,
    device_scale: DeviceScale,
    pending_high_surrogate: Option<u16>,
    active: bool,
    renderer: Option<HwndRenderer>,
    last_scene: Option<Arc<SceneSnapshot>>,
    pending_render: Option<PendingRender>,
}

type PendingRender = (
    Arc<SceneSnapshot>,
    SyncSender<Result<(), PlatformOperationError>>,
);

struct NativeWindow {
    handle: HWND,
    _state: Box<WindowState>,
    initial_info: WindowOpenInfo,
}

fn worker_main(receiver: Receiver<Command>, ready: SyncSender<()>, wake_event: usize) {
    let mut message = MSG::default();
    // SAFETY: this call initializes the thread's message queue before the worker is published.
    unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_NOREMOVE) };
    let _ = ready.send(());
    let event_handle = wake_event as _;
    let mut window: Option<NativeWindow> = None;

    'worker: loop {
        // SAFETY: the worker owns event_handle and waits for its command signal or thread messages.
        let wait = unsafe { MsgWaitForMultipleObjects(1, &event_handle, 0, INFINITE, u32::MAX) };
        if wait == WAIT_FAILED {
            break;
        }

        if wait == WAIT_OBJECT_0 {
            // SAFETY: event_handle is a valid manual-reset event created by Worker::start.
            if unsafe { ResetEvent(event_handle) } == 0 {
                break;
            }
            loop {
                match receiver.try_recv() {
                    Ok(Command::Open {
                        generation,
                        sink,
                        close_request_sink,
                        input_sink,
                        response,
                    }) => {
                        let result = if window.is_some() {
                            Err(PlatformOperationError)
                        } else {
                            match open_native_window(
                                generation,
                                sink,
                                close_request_sink,
                                input_sink,
                            ) {
                                Ok(opened) => {
                                    let info = opened.initial_info;
                                    window = Some(opened);
                                    Ok(info)
                                }
                                Err(error) => Err(error),
                            }
                        };
                        let _ = response.send(result);
                    }
                    Ok(Command::Close { response }) => {
                        let result = close_native_window(&mut window);
                        let _ = response.send(result);
                    }
                    Ok(Command::SubmitFrame { scene, response }) => {
                        let result = if let Some(opened) = window.as_mut() {
                            let state = &mut opened._state;
                            if state.pending_render.is_some() {
                                Err(PlatformOperationError)
                            } else {
                                state.pending_render = Some((scene, response));
                                // SAFETY: opened.handle is live and owned by this worker thread.
                                if unsafe { InvalidateRect(opened.handle, null(), 0) } == 0 {
                                    if let Some((_, response)) = state.pending_render.take() {
                                        let _ = response.send(Err(PlatformOperationError));
                                    }
                                } else {
                                    // SAFETY: invalidation above guarantees an update region.
                                    let updated = unsafe { UpdateWindow(opened.handle) };
                                    if updated == 0 {
                                        if let Some((_, response)) = state.pending_render.take() {
                                            let _ = response.send(Err(PlatformOperationError));
                                        }
                                    }
                                }
                                continue;
                            }
                        } else {
                            Err(PlatformOperationError)
                        };
                        let _ = response.send(result);
                    }
                    Ok(Command::ResolveText { request, response }) => {
                        let renderer = window
                            .as_mut()
                            .and_then(|opened| opened._state.renderer.as_mut())
                            .map(|renderer| renderer as &mut dyn WorkerTextResolver);
                        let result = resolve_text_command(renderer, &request);
                        let _ = response.send(result);
                    }
                    Ok(Command::Shutdown) => {
                        let _ = close_native_window(&mut window);
                        break 'worker;
                    }
                    Err(TryRecvError::Empty) => break,
                    Err(TryRecvError::Disconnected) => {
                        let _ = close_native_window(&mut window);
                        break 'worker;
                    }
                }
            }
            continue;
        }

        if wait == WAIT_OBJECT_0 + 1 {
            loop {
                // SAFETY: message is writable and this worker owns the thread message queue.
                if unsafe { PeekMessageW(&mut message, null_mut(), 0, 0, PM_REMOVE) } == 0 {
                    break;
                }
                // SAFETY: message was retrieved from this thread's queue and is valid to dispatch.
                unsafe {
                    TranslateMessage(&message);
                    DispatchMessageW(&message);
                }
            }
        }
    }

    // SAFETY: the worker is the sole owner of the event handle after startup.
    unsafe { CloseHandle(event_handle) };
}

fn open_native_window(
    generation: u64,
    sink: Option<Arc<dyn PlatformWindowClientAreaSink>>,
    close_request_sink: Option<Arc<dyn PlatformWindowCloseRequestSink>>,
    input_sink: Option<Arc<dyn PlatformWindowInputSink>>,
) -> Result<NativeWindow, PlatformOperationError> {
    register_window_class()?;
    let mut state = Box::new(WindowState {
        generation,
        sink,
        close_request_sink,
        input_sink,
        device_scale: DeviceScale::new(1.0, 1.0).map_err(|_| PlatformOperationError)?,
        pending_high_surrogate: None,
        active: false,
        renderer: None,
        last_scene: None,
        pending_render: None,
    });
    let state_pointer = (&mut *state) as *mut WindowState;
    // SAFETY: pointers refer to static UTF-16 strings or state retained by NativeWindow. The
    // Win32 message loop and the handle are owned by this worker thread.
    let handle = unsafe {
        CreateWindowExW(
            0,
            CLASS_NAME_NUL.as_ptr(),
            WINDOW_TITLE.as_ptr(),
            WS_OVERLAPPEDWINDOW,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            CW_USEDEFAULT,
            null_mut(),
            null_mut(),
            GetModuleHandleW(null()),
            state_pointer.cast::<c_void>(),
        )
    };
    if handle.is_null() {
        return Err(PlatformOperationError);
    }

    let (width_px, height_px) = match client_area_pixels(handle) {
        Ok(size) => size,
        Err(error) => {
            // SAFETY: the HWND is live and owned by this worker thread.
            unsafe { DestroyWindow(handle) };
            return Err(error);
        }
    };
    match HwndRenderer::new(
        windows::Win32::Foundation::HWND(handle),
        width_px,
        height_px,
    ) {
        Ok(renderer) => state.renderer = Some(renderer),
        Err(error) => {
            // SAFETY: the HWND is live and owned by this worker thread.
            unsafe { DestroyWindow(handle) };
            return Err(error);
        }
    }

    // SAFETY: handle is a live HWND created and owned by this worker thread.
    let dpi = unsafe { GetDpiForWindow(handle) };
    if dpi == 0 {
        // SAFETY: handle is still a live HWND owned by this worker thread.
        unsafe { DestroyWindow(handle) };
        return Err(PlatformOperationError);
    }
    let scale = f64::from(dpi) / 96.0;
    state.device_scale = DeviceScale::new(scale, scale).map_err(|_| PlatformOperationError)?;

    // SAFETY: handle is a live top-level Window created by this worker thread.
    unsafe {
        ShowWindow(handle, SW_SHOW);
        UpdateWindow(handle);
    }
    let initial_info = match window_open_info(handle) {
        Ok(info) => info,
        Err(error) => {
            // SAFETY: handle is live and owned by this worker thread.
            unsafe { DestroyWindow(handle) };
            return Err(error);
        }
    };
    Ok(NativeWindow {
        handle,
        _state: state,
        initial_info,
    })
}

fn close_native_window(window: &mut Option<NativeWindow>) -> Result<(), PlatformOperationError> {
    let Some(opened) = window.as_ref() else {
        return Ok(());
    };
    // SAFETY: the current worker created and owns this Window handle.
    if unsafe { DestroyWindow(opened.handle) } == 0 {
        return Err(PlatformOperationError);
    }
    let _ = window.take();
    Ok(())
}

fn client_area(handle: HWND) -> Result<WindowClientArea, PlatformOperationError> {
    let (width_px, height_px) = client_area_pixels(handle)?;
    // SAFETY: handle is a live Window on its owning thread.
    let dpi = unsafe { GetDpiForWindow(handle) };
    if dpi == 0 {
        return Err(PlatformOperationError);
    }
    let scale = 96.0 / f64::from(dpi);
    Ok(WindowClientArea {
        width_dip: f64::from(width_px) * scale,
        height_dip: f64::from(height_px) * scale,
    })
}

fn window_open_info(handle: HWND) -> Result<WindowOpenInfo, PlatformOperationError> {
    let client_area = client_area(handle)?;
    // SAFETY: handle is a live Window on its owning thread.
    let dpi = unsafe { GetDpiForWindow(handle) };
    if dpi == 0 {
        return Err(PlatformOperationError);
    }
    let scale = f64::from(dpi) / 96.0;
    let device_scale = DeviceScale::new(scale, scale).map_err(|_| PlatformOperationError)?;
    Ok(WindowOpenInfo {
        client_area,
        device_scale,
    })
}

fn client_area_pixels(handle: HWND) -> Result<(u32, u32), PlatformOperationError> {
    // SAFETY: RECT is a plain Win32 output struct whose all-zero state is valid before the call.
    let mut rect = unsafe { zeroed() };
    // SAFETY: handle is live on its owning thread and rect is writable.
    if unsafe { GetClientRect(handle, &mut rect) } == 0 {
        return Err(PlatformOperationError);
    }
    Ok((
        (rect.right - rect.left).max(0) as u32,
        (rect.bottom - rect.top).max(0) as u32,
    ))
}

fn register_window_class() -> Result<(), PlatformOperationError> {
    // SAFETY: a null module name requests this process's module handle.
    let instance = unsafe { GetModuleHandleW(null()) };
    if instance.is_null() {
        return Err(PlatformOperationError);
    }
    let class = WNDCLASSW {
        style: CS_HREDRAW | CS_VREDRAW,
        lpfnWndProc: Some(window_proc),
        cbClsExtra: 0,
        cbWndExtra: 0,
        hInstance: instance,
        hIcon: null_mut(),
        hCursor: null_mut(),
        hbrBackground: null_mut(),
        lpszMenuName: null(),
        lpszClassName: CLASS_NAME_NUL.as_ptr(),
    };
    // SAFETY: class has static strings and a valid system callback.
    if unsafe { RegisterClassW(&class) } != 0 {
        Ok(())
    } else {
        // SAFETY: GetLastError is thread-local and safe immediately after RegisterClassW failure.
        if unsafe { GetLastError() } == 1410 {
            // ERROR_CLASS_ALREADY_EXISTS is expected for a later Window or concurrent peer.
            Ok(())
        } else {
            Err(PlatformOperationError)
        }
    }
}

unsafe extern "system" fn window_proc(
    handle: HWND,
    message: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if message == WM_NCCREATE {
        let create = lparam as *const CREATESTRUCTW;
        if !create.is_null() {
            // SAFETY: WM_NCCREATE supplies a live CREATESTRUCTW whose lpCreateParams is the
            // WindowState allocation retained by NativeWindow for the full HWND lifetime.
            let state = unsafe { (*create).lpCreateParams };
            // SAFETY: this callback owns the just-created HWND and stores the opaque state pointer.
            unsafe { SetWindowLongPtrW(handle, GWLP_USERDATA, state as isize) };
        }
        // SAFETY: delegates default non-client initialization to Win32.
        return unsafe { DefWindowProcW(handle, message, wparam, lparam) };
    }

    if message == WM_CLOSE {
        // Post only a generation-tagged intent. The Core decides whether and when to enqueue the
        // worker close command; this message handler never waits for the UI dispatcher.
        // SAFETY: GWLP_USERDATA is either null or the state pointer installed at WM_NCCREATE.
        let state = unsafe { GetWindowLongPtrW(handle, GWLP_USERDATA) } as *mut WindowState;
        let (generation, sink) = if !state.is_null() {
            // SAFETY: state is retained by NativeWindow until DestroyWindow returns on this thread.
            let state = unsafe { &*state };
            (Some(state.generation), state.close_request_sink.as_ref())
        } else {
            (None, None)
        };
        return handle_close_message(generation, sink);
    }

    // Native input is normalized and copied into the Core-owned dispatcher sink. The sink only
    // enqueues work; it never runs Core routing or application callbacks on this worker thread.
    // SAFETY: this WndProc invocation belongs to handle's owning thread.
    let state_pointer = unsafe { GetWindowLongPtrW(handle, GWLP_USERDATA) } as *mut WindowState;
    if !state_pointer.is_null() {
        // SAFETY: WindowState is retained by NativeWindow until DestroyWindow returns on this
        // message thread, and every WndProc invocation runs on that same owner thread.
        let state = unsafe { &mut *state_pointer };
        match message {
            WM_KEYDOWN | WM_SYSKEYDOWN | WM_KEYUP | WM_SYSKEYUP => {
                let down = matches!(message, WM_KEYDOWN | WM_SYSKEYDOWN);
                let modifiers = current_modifiers();
                let event = map_key_message(down, lparam, modifiers);
                let _ = post_input_event(state, event);
                if matches!(message, WM_SYSKEYDOWN | WM_SYSKEYUP) {
                    // System keys must retain native menu/activation behavior.
                    // SAFETY: native system-key messages require the standard default procedure.
                    return unsafe { DefWindowProcW(handle, message, wparam, lparam) };
                }
            }
            WM_CHAR => {
                if let Some(event) =
                    map_utf16_char(wparam as u16, &mut state.pending_high_surrogate)
                {
                    let _ = post_input_event(state, event);
                }
                return 0;
            }
            WM_DEADCHAR | WM_SYSCHAR | WM_SYSDEADCHAR => {
                // Text input is sourced only from WM_CHAR; IME result strings use a later adapter.
            }
            WM_LBUTTONDOWN | WM_LBUTTONUP => {
                let (x_px, y_px) = signed_client_pixels(lparam);
                if let Some(event) = map_pointer_button(
                    message == WM_LBUTTONDOWN,
                    PointerDevice::Mouse,
                    true,
                    true,
                    (x_px, y_px),
                    state.device_scale,
                ) {
                    let _ = post_input_event(state, event);
                }
            }
            WM_POINTERDOWN | WM_POINTERUP => {
                let pointer_id = (wparam & 0xffff) as u32;
                let mut pointer_type = 0;
                // SAFETY: pointer_type is writable; pointer_id comes from the WM_POINTER message.
                if unsafe { GetPointerType(pointer_id, &mut pointer_type) } != 0 {
                    let device = match pointer_type {
                        PT_TOUCH => PointerDevice::Touch,
                        PT_PEN => PointerDevice::Pen,
                        _ => PointerDevice::Other,
                    };
                    let flags = ((wparam >> 16) & 0xffff) as u32;
                    if matches!(device, PointerDevice::Touch | PointerDevice::Pen)
                        && flags & POINTER_MESSAGE_FLAG_PRIMARY != 0
                        && flags & POINTER_MESSAGE_FLAG_FIRSTBUTTON != 0
                    {
                        let mut info = POINTER_INFO::default();
                        // SAFETY: info is a valid output buffer and pointer_id belongs to this event.
                        if unsafe { GetPointerInfo(pointer_id, &mut info) } != 0 {
                            let mut point = POINT {
                                x: info.ptPixelLocation.x,
                                y: info.ptPixelLocation.y,
                            };
                            // SAFETY: handle is the live window for this callback and point is writable.
                            if unsafe { ScreenToClient(handle, &mut point) } != 0 {
                                if let Some(event) = map_pointer_button(
                                    message == WM_POINTERDOWN,
                                    device,
                                    true,
                                    true,
                                    (point.x, point.y),
                                    state.device_scale,
                                ) {
                                    let _ = post_input_event(state, event);
                                    return 0;
                                }
                            }
                        }
                    }
                }
            }
            WM_ACTIVATE => {
                let active = (wparam & 0xffff) as u32 != WA_INACTIVE;
                post_activation_change(state, active);
            }
            WM_KILLFOCUS => post_activation_change(state, false),
            WM_SETFOCUS => post_activation_change(state, true),
            _ => {}
        }
    }

    if message == WM_SIZE {
        // SAFETY: GWLP_USERDATA is either null or the state installed at WM_NCCREATE.
        let state = unsafe { GetWindowLongPtrW(handle, GWLP_USERDATA) } as *mut WindowState;
        if !state.is_null() {
            // SAFETY: state is retained until DestroyWindow returns; this is its owner thread.
            let state = unsafe { &mut *state };
            if let Some(renderer) = &mut state.renderer {
                if let Ok((width, height)) = client_area_pixels(handle) {
                    if renderer.resize(width, height).is_err() {
                        state.renderer = None;
                    }
                }
            }
            if let Ok(area) = client_area(handle) {
                if let Some(sink) = &state.sink {
                    let sink = Arc::clone(sink);
                    let generation = state.generation;
                    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                        let _ = sink.client_area_changed(generation, area);
                    }));
                }
            }
        }
        // SAFETY: delegates default non-client behavior; WM_SIZE is otherwise fully observed.
        return unsafe { DefWindowProcW(handle, message, wparam, lparam) };
    }

    if message == WM_PAINT {
        let mut paint = PAINTSTRUCT::default();
        // SAFETY: BeginPaint/EndPaint validate this Window's update region on its owner thread.
        let context = unsafe { BeginPaint(handle, &mut paint) };
        // SAFETY: this balances the successful BeginPaint call above.
        unsafe { EndPaint(handle, &paint) };
        // SAFETY: GWLP_USERDATA is either null or the state installed at WM_NCCREATE.
        let state = unsafe { GetWindowLongPtrW(handle, GWLP_USERDATA) } as *mut WindowState;
        if !state.is_null() {
            // SAFETY: state remains owned by NativeWindow through this window procedure call.
            let state = unsafe { &mut *state };
            let pending = state.pending_render.take();
            let scene = pending
                .as_ref()
                .map(|(scene, _)| Arc::clone(scene))
                .or_else(|| state.last_scene.clone());
            let result = match (state.renderer.as_mut(), scene.as_ref()) {
                (Some(renderer), Some(scene)) => {
                    renderer.render(scene).map_err(|_| PlatformOperationError)
                }
                (Some(_), None) => Ok(()),
                (None, _) => Err(PlatformOperationError),
            };
            if result.is_ok() {
                if let Some((scene, _)) = &pending {
                    state.last_scene = Some(Arc::clone(scene));
                }
            }
            if let Some((_, response)) = pending {
                let _ = response.send(result);
            }
        }
        let _ = context;
        return 0;
    }

    if message == WM_NCDESTROY {
        // SAFETY: this clears a stale opaque pointer before the final default processing.
        unsafe { SetWindowLongPtrW(handle, GWLP_USERDATA, 0) };
    }

    // SAFETY: all messages not handled above follow the standard Win32 default procedure.
    unsafe { DefWindowProcW(handle, message, wparam, lparam) }
}

fn post_close_intent(
    generation: u64,
    sink: Option<&Arc<dyn PlatformWindowCloseRequestSink>>,
) -> Result<(), PlatformOperationError> {
    let sink = sink.ok_or(PlatformOperationError)?;
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sink.close_requested(generation)
    }))
    .unwrap_or(Err(PlatformOperationError))
}

fn handle_close_message(
    generation: Option<u64>,
    sink: Option<&Arc<dyn PlatformWindowCloseRequestSink>>,
) -> LRESULT {
    if let Some(generation) = generation {
        // Native default processing would destroy the HWND before Core accepts the request.
        // Even a failed post keeps it alive; explicit close/shutdown commands own destruction.
        let _ = post_close_intent(generation, sink);
    }
    0
}

fn post_input_event(state: &WindowState, event: PlatformWindowInputEvent) -> bool {
    let Some(sink) = state.input_sink.as_ref() else {
        return false;
    };
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        sink.post(state.generation, event)
    }))
    .is_ok_and(|result| result.is_ok())
}

fn post_activation_change(state: &mut WindowState, active: bool) {
    if state.active == active {
        return;
    }
    state.active = active;
    if !active {
        state.pending_high_surrogate = None;
    }
    let _ = post_input_event(
        state,
        PlatformWindowInputEvent::ActivationChanged { active },
    );
}

fn current_modifiers() -> u32 {
    // SAFETY: GetKeyState reads the current thread's keyboard state and accepts these virtual keys.
    let pressed = |key| unsafe { GetKeyState(key as i32) < 0 };
    modifier_mask(
        pressed(VK_SHIFT),
        pressed(VK_CONTROL),
        pressed(VK_MENU),
        pressed(VK_LWIN) || pressed(VK_RWIN),
    )
}

#[cfg(test)]
mod tests {
    use super::{WorkerTextResolver, handle_close_message, resolve_text_command};
    use freemvvm_core::abi_support::{
        PlatformOperationError, PlatformWindowCloseRequestSink, ResolvedTextSnapshot, TextMetrics,
        TextRequest, TextServiceError,
    };
    use std::sync::{Arc, Mutex};

    struct FakeResolver {
        requests: Arc<Mutex<Vec<(u64, String)>>>,
        result: Result<ResolvedTextSnapshot, TextServiceError>,
    }

    impl WorkerTextResolver for FakeResolver {
        fn resolve_text(
            &mut self,
            request: &TextRequest,
        ) -> Result<ResolvedTextSnapshot, TextServiceError> {
            self.requests
                .lock()
                .unwrap()
                .push((request.revision, request.text.to_string()));
            self.result.clone()
        }
    }

    struct RecordingCloseSink {
        requests: Arc<Mutex<Vec<u64>>>,
        result: Result<(), PlatformOperationError>,
    }

    impl PlatformWindowCloseRequestSink for RecordingCloseSink {
        fn close_requested(&self, generation: u64) -> Result<(), PlatformOperationError> {
            self.requests.lock().unwrap().push(generation);
            self.result
        }
    }

    #[test]
    fn platform_windows_close_intent_posts_generation_and_leaves_close_to_core_command() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let sink: Arc<dyn PlatformWindowCloseRequestSink> = Arc::new(RecordingCloseSink {
            requests: Arc::clone(&requests),
            result: Ok(()),
        });

        assert_eq!(handle_close_message(Some(41), Some(&sink)), 0);
        assert_eq!(*requests.lock().unwrap(), [41]);
        assert_eq!(handle_close_message(Some(42), None), 0);
        assert_eq!(handle_close_message(None, None), 0);
    }

    #[test]
    fn platform_windows_close_intent_post_failure_keeps_default_close_suppressed() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let sink: Arc<dyn PlatformWindowCloseRequestSink> = Arc::new(RecordingCloseSink {
            requests: Arc::clone(&requests),
            result: Err(PlatformOperationError),
        });

        assert_eq!(handle_close_message(Some(43), Some(&sink)), 0);
        assert_eq!(*requests.lock().unwrap(), [43]);
    }

    #[test]
    fn platform_windows_text_flow_worker_dispatches_revision_to_window_resolver() {
        let requests = Arc::new(Mutex::new(Vec::new()));
        let snapshot = ResolvedTextSnapshot::new(
            29,
            TextMetrics::new(31.0, 14.0).unwrap(),
            Arc::<[freemvvm_core::abi_support::ResolvedGlyphRun]>::from([]),
        );
        let mut resolver = FakeResolver {
            requests: Arc::clone(&requests),
            result: Ok(snapshot.clone()),
        };

        let result = resolve_text_command(
            Some(&mut resolver),
            &TextRequest::platform_default(29, "worker request"),
        )
        .unwrap();

        assert_eq!(result, snapshot);
        assert_eq!(
            *requests.lock().unwrap(),
            [(29, "worker request".to_owned())]
        );
        assert_eq!(
            resolve_text_command(None, &TextRequest::platform_default(30, "closed")),
            Err(TextServiceError::Unavailable)
        );
        let mut failed_resolver = FakeResolver {
            requests: Arc::clone(&requests),
            result: Err(TextServiceError::ShapingFailed),
        };
        assert_eq!(
            resolve_text_command(
                Some(&mut failed_resolver),
                &TextRequest::platform_default(31, "invalid"),
            ),
            Err(TextServiceError::ShapingFailed)
        );
    }
}
