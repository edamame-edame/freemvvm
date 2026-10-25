use freemvvm::{UiRuntime, Window};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = UiRuntime::try_create()?;
    {
        let ui = runtime.ui_context()?;
        let mut window = Window::create(&ui)?;
        let mut close_requests = window.on_close_request(&ui, |request| {
            // This callback can defer using only its borrowed view; it need not capture `ui`.
            let _deferred_close = request.defer();
        })?;
        close_requests.cancel(&ui)?;
        close_requests.close()?;
        window.close()?;
    }
    runtime.close()?;
    Ok(())
}
