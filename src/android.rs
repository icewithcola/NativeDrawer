use jni::{
    JNIEnv, JavaVM,
    objects::{GlobalRef, JObject, JValue},
};

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
        AndroidEnv::run_on_main_thread(|| {
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
    fn run_on_main_thread<F, R>(func: F)
    where
        F: Fn() -> R + Sized,
        R: Sized,
    {
    }

    // fn get_main_looper(&self) -> JObject {
    //     let jni_env = &mut self.vm.get_env().unwrap();
    //     jni_env
    //         .call_static_method(
    //             "android/os/Looper",
    //             "getMainLooper",
    //             "()Landroid/os/Looper;",
    //             &[],
    //         )
    //         .unwrap()
    //         .l()
    //         .unwrap()
    // }
}
