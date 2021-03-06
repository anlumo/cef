#[cfg(windows)]
use winapi::um::winuser::{WS_CLIPCHILDREN, WS_CLIPSIBLINGS, WS_OVERLAPPEDWINDOW, WS_VISIBLE};
use cef::{
    app::{App, AppCallbacks},
    browser::{Browser, BrowserSettings},
    browser_host::BrowserHost,
    client::{
        Client, ClientCallbacks,
        life_span_handler::{LifeSpanHandler, LifeSpanHandlerCallbacks}
    },
    settings::{Settings},
    window::WindowInfo,
    logging::Logger,
};
use log::info;

pub struct AppCallbacksImpl {}

impl AppCallbacks for AppCallbacksImpl {}

pub struct ClientCallbacksImpl {
    life_span_handler: LifeSpanHandler,
}

impl ClientCallbacks for ClientCallbacksImpl {
    fn get_life_span_handler(&self) -> Option<LifeSpanHandler> {
        Some(self.life_span_handler.clone())
    }
}

pub struct LifeSpanHandlerImpl {}

impl LifeSpanHandlerCallbacks for LifeSpanHandlerImpl {
    fn on_before_close(&self, _browser: Browser) {
        cef::quit_message_loop().unwrap();
    }
}

fn main() {
    let app = App::new(AppCallbacksImpl {});
    let result = cef::execute_process(Some(app.clone()), None);
    if result >= 0 {
        std::process::exit(result);
    }

    let settings = Settings::new()
        .log_severity(cef::settings::LogSeverity::Info);

    let context = cef::Context::initialize(&settings, Some(app), None).unwrap();
    let mut logger_builder = Logger::builder();
    logger_builder.level(log::LevelFilter::Info);
    let logger = Box::new(logger_builder.build());
    log::set_boxed_logger(logger).map(|()| log::set_max_level(log::LevelFilter::Info)).unwrap();
    info!("Startup"); // This is the earliest you can use logging!

    let mut window_info = WindowInfo::new();
    #[cfg(windows)] {
        window_info.platform_specific.style = WS_OVERLAPPEDWINDOW | WS_CLIPCHILDREN | WS_CLIPSIBLINGS | WS_VISIBLE;
    }
    window_info.window_name = "cefsimple Rust example".into();
    window_info.width = 500;
    window_info.height = 500;
    let browser_settings = BrowserSettings::new();

    let client = Client::new(ClientCallbacksImpl {
        life_span_handler: LifeSpanHandler::new(LifeSpanHandlerImpl {})
    });

    info!("Opening browser window");

    let _browser = BrowserHost::create_browser_sync(
        &window_info,
        client,
        "https://webkit.org/blog-files/3d-transforms/morphing-cubes.html",
        &browser_settings,
        None,
        None,
        &context,
    );

    info!("Running message loop");

    context.run_message_loop();

    info!("Quit");
}
