use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JObject, JValue},
};
use ndk::looper::ThreadLooper;

#[derive(Debug)]
pub struct AndroidEnv {
    vm: JavaVM,
    activity: GlobalRef,
}

impl AndroidEnv {
    pub fn new(vm: JavaVM, activity: JObject) -> Self {
        let activity = JNIEnv::new_global_ref(&vm.get_env().unwrap(), activity).unwrap();
        AndroidEnv { vm, activity }
    }

    pub fn show_toast(&self, message: &str) {
        self.run_on_main_thread(|| {
            let jni_env = &mut self.vm.get_env().unwrap();
            jni_env
                .call_static_method(
                    "android/widget/Toast",
                    "makeText",
                    "(Landroid/content/Context;Ljava/lang/CharSequence;I)Landroid/widget/Toast;",
                    &[
                        JValue::Object(&self.activity),
                        JValue::Object(&jni_env.new_string(message).unwrap()),
                        JValue::Int(300),
                    ],
                )
                .unwrap();
        });
    }

    /// JNI main thread is also rust main thread
    /// the thread is always called `native`
    fn run_on_main_thread<F, R>(&self, func: F)
    where
        F: Fn() -> R + Sized,
        R: Sized,
    {
        let jni_env = &mut self.vm.get_env().unwrap();
        let main_handler = self.get_main_handler();
        let _closure_ptr: *mut F = Box::into_raw(Box::new(func));
        
        let runnable = jni_env
            .new_object("java/lang/Runnable", "()V", &[])
            .unwrap();
        jni_env
            .call_method(
                main_handler,
                "post",
                "(Ljava/lang/Runnable;)Z",
                &[JValue::Object(&runnable)],
            )
            .unwrap();
    }

    /// Gets the main handler, this is required for running on the main thread
    fn get_main_handler(&'_ self) -> JObject<'_> {
        let jni_env = &mut self.vm.get_env().unwrap();
        let main_looper = jni_env
            .call_static_method(
                "android/os/Looper",
                "getMainLooper",
                "()Landroid/os/Looper;",
                &[],
            )
            .unwrap()
            .l()
            .unwrap();
        let handler_class = jni_env.find_class("android/os/Handler").unwrap();
        jni_env
            .new_object(
                handler_class,
                "(Landroid/os/Looper;)V",
                &[JValue::Object(&main_looper)],
            )
            .unwrap()
    }
}
