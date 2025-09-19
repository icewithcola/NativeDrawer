use jni::{JavaVM, objects::JObject};
use winit::{event_loop::EventLoop, platform::android::activity::AndroidApp};

use crate::android::AndroidEnv;

mod android;
mod app;
mod user_input;


/// The entry point of the application.
/// 
/// Never mangle or call this function, android will call it.
#[unsafe(no_mangle)]
fn android_main(app: AndroidApp) {
    use winit::platform::android::EventLoopBuilderExtAndroid;

    let vm = unsafe { JavaVM::from_raw(app.vm_as_ptr() as *mut _) };
    let activity = unsafe { JObject::from_raw(app.activity_as_ptr() as *mut _) };
    let android_env = if !(vm.is_err() || activity.is_null()) {
        Some(AndroidEnv::new(vm.unwrap(), activity))
    } else {
        None
    };

    android_logger::init_once(
        android_logger::Config::default().with_max_level(log::LevelFilter::Info),
    );
    let event_loop: EventLoop<AndroidApp> = EventLoop::<AndroidApp>::with_user_event()
        .with_android_app(app)
        .build()
        .unwrap();
    app::run(event_loop, android_env);
}
