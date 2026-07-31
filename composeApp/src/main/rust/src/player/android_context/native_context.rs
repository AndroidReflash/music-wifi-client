use std::sync::OnceLock;

use jni::objects::{GlobalRef, JObject};
use jni::sys::{jboolean, JNI_FALSE, JNI_TRUE};
use jni::JNIEnv;

// Глобальная ссылка живёт всё время процесса: ndk_context хранит голый
// указатель на этот объект, и сборщик мусора не должен его переместить.
static CONTEXT: OnceLock<GlobalRef> = OnceLock::new();

// Имя символа JNI собирается из пакета, класса и метода на стороне Kotlin:
// music.player.NativeContext.initialize. Любое из трёх переименовывается
// только вместе с этой строкой.
#[no_mangle]
pub extern "system" fn Java_music_player_NativeContext_initialize(
    env: JNIEnv,
    _this: JObject,
    context: JObject,
) -> jboolean {
    if CONTEXT.get().is_some() {
        return JNI_TRUE;
    }

    let vm = match env.get_java_vm() {
        Ok(vm) => vm,
        Err(_) => return JNI_FALSE,
    };
    let global = match env.new_global_ref(context) {
        Ok(global) => global,
        Err(_) => return JNI_FALSE,
    };

    if CONTEXT.set(global).is_err() {
        return JNI_TRUE;
    }
    let stored = CONTEXT.get().expect("ссылка только что положена в CONTEXT");

    // Безопасность: указатели получены из живого JNIEnv, глобальная ссылка
    // лежит в CONTEXT до конца процесса, и вызов здесь ровно один — его
    // стережёт OnceLock выше.
    unsafe {
        ndk_context::initialize_android_context(
            vm.get_java_vm_pointer().cast(),
            stored.as_obj().as_raw().cast(),
        );
    }

    JNI_TRUE
}
