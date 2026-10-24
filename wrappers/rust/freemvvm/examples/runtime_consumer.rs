use freemvvm::{STATUS_UNSUPPORTED, ShutdownMode, UiRuntime};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut runtime = UiRuntime::try_create()?;
    let request_result = {
        let ui = runtime.ui_context()?;
        assert!(runtime.is_ui_thread()?);
        runtime.request_shutdown(&ui, ShutdownMode::Async)
    };

    match request_result {
        Ok(mut request) => {
            let snapshot = request.query()?;
            println!("shutdown request state: {:?}", snapshot.known_state());
            request.close()?;
        }
        Err(error) if error.raw_status() == STATUS_UNSUPPORTED => {
            // This consumer has no host loop bridge, so no shutdown receipt is created.
            println!("shutdown request requires a registered host loop");
        }
        Err(error) => return Err(Box::new(error)),
    }

    runtime.close()?;
    Ok(())
}
