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

    /// getSystemService
    fn get_system_service(&'_ self, service_name: &str) -> Result<JObject<'_>, jni::errors::Error> {
        let env = &mut self.vm.get_env().unwrap();
        let binding = env.new_string(service_name).unwrap();
        let service_name = JValue::Object(&binding);
        Ok(env
            .call_method(
                self.activity.clone(),
                "getSystemService",
                "(Ljava/lang/String;)Ljava/lang/Object;",
                &[service_name],
            )?
            .l()?)
    }

    /// vibrate
    pub fn vibrate(&self, duration: i64) -> Result<(), jni::errors::Error> {
        let env = &mut self.vm.get_env().unwrap();
        let service = self.get_system_service("vibrator_manager")?;

        let vibrator = env
            .call_method(
                service,
                "getDefaultVibrator",
                "()Landroid/os/Vibrator;",
                &[],
            )?
            .l()?;

        let vibration_effect = env
            .call_static_method(
                "android/os/VibrationEffect",
                "createOneShot",
                "(JI)Landroid/os/VibrationEffect;",
                &[JValue::Long(duration), JValue::Int(-1)],
            )?
            .l()?;

        env.call_method(
            vibrator,
            "vibrate",
            "(Landroid/os/VibrationEffect;)V",
            &[JValue::Object(&vibration_effect)],
        )?;

        Ok(())
    }
}
