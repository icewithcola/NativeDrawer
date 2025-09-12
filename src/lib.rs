use winit::{event_loop::EventLoop, platform::android::activity::AndroidApp};

mod app;
mod user_input;

#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    android_logger::init_once(android_logger::Config::default().with_max_level(log::LevelFilter::Info));
    let event_loop: EventLoop<AndroidApp> = EventLoop::<AndroidApp>::with_user_event().with_android_app(app).build().unwrap();
    app::run(event_loop);
}