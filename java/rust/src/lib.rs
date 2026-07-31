use jni::errors::Error;
use jni::objects::{JObject, JString};
use jni::strings::JNIString;
use jni::{
    EnvUnowned, JValue, jni_sig, jni_str,
    objects::{JByteArray, JClass},
    sys::jbyteArray,
};
use packobf::options::{Compression, Options, ShaderCompression};
use packobf::{LogMessage, Progress, process_zip};
use tokio::sync::watch;

#[unsafe(no_mangle)]
pub extern "system" fn Java_dev_misieur_packobf_Native_optimizeZip<'caller>(
    mut unowned_env: EnvUnowned<'caller>,
    _class: JClass<'caller>,
    input: JByteArray<'caller>,
    options: JObject<'caller>,
    log_callback: JObject<'caller>,
    progress_callback: JObject<'caller>,
    cache_file: JString<'caller>,
) -> jbyteArray {
    let outcome = unowned_env.with_env(|env| -> Result<_, jni::errors::Error> {
        let input: Vec<u8> = env.convert_byte_array(input)?;

        let cache_file: Option<String> = if cache_file.is_null() {
            None
        } else {
            Some(cache_file.mutf8_chars(env)?.to_string())
        };

        let options = if options.is_null() {
            Options::fastest()
        } else {
            let comp_val = env
                .get_field(&options, jni_str!("compression"), jni_sig!("I"))?
                .i()?;
            let shader_comp_val = env
                .get_field(&options, jni_str!("shaderCompression"), jni_sig!("I"))?
                .i()?;

            Options {
                compression: match comp_val {
                    0 => Compression::Fastest,
                    1 => Compression::Fast,
                    2 => Compression::Normal,
                    _ => Compression::Best,
                },
                shader_compression: match shader_comp_val {
                    0 => ShaderCompression::None,
                    1 => ShaderCompression::Minify,
                    _ => ShaderCompression::MinifyAndObfuscate,
                },
                rename_files: env
                    .get_field(&options, jni_str!("renameFiles"), jni_sig!("Z"))?
                    .z()?,
                block_unzipping: env
                    .get_field(&options, jni_str!("blockUnzipping"), jni_sig!("Z"))?
                    .z()?,
                corrupt_png_files: env
                    .get_field(&options, jni_str!("corruptPngFiles"), jni_sig!("Z"))?
                    .z()?,
                num_threads: Some(
                    env.get_field(&options, jni_str!("numThreads"), jni_sig!("I"))?
                        .i()? as usize,
                ),
            }
        };

        let (log_tx, mut log_rx) = tokio::sync::mpsc::unbounded_channel::<LogMessage>();
        let (prog_tx, mut prog_rx) = watch::channel(Progress::Idle);

        let jvm = env.get_java_vm()?;

        if !log_callback.is_null() {
            let global_log_cb = env.new_global_ref(&log_callback)?;
            let jvm_clone = jvm.clone();

            std::thread::spawn(move || {
                let _ = jvm_clone.attach_current_thread(|env| -> Result<(), jni::errors::Error> {
                    while let Some(msg) = log_rx.blocking_recv() {
                        let _ = env.with_local_frame(16, |env| {
                            let level_int = msg.level as i32;
                            if let Ok(j_msg) = env.new_string(msg.message) {
                                let _ = env.call_method(
                                    &global_log_cb,
                                    jni_str!("onLog"),
                                    jni_sig!("(ILjava/lang/String;)V"),
                                    &[JValue::Int(level_int), JValue::Object(&j_msg.into())],
                                );
                            }
                            Ok::<(), jni::errors::Error>(())
                        });
                    }
                    Ok(())
                });
            });
        }

        if !progress_callback.is_null() {
            let global_prog_cb = env.new_global_ref(&progress_callback)?;
            let jvm_clone = jvm.clone();

            std::thread::spawn(move || {
                let _ = jvm_clone.attach_current_thread(|env| -> Result<(), jni::errors::Error> {
                    let rt = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .unwrap();

                    loop {
                        let changed = rt.block_on(async { prog_rx.changed().await.is_ok() });

                        if !changed {
                            break;
                        }

                        let prog = prog_rx.borrow().clone();

                        let (state, current, total, cur_str) = match prog {
                            Progress::Idle => (0, 0, 0, None),
                            Progress::ReadingZip {
                                current: c,
                                total: t,
                            } => (1, c as i32, t as i32, None),
                            Progress::Parsing { current: s } => (2, 0, 0, Some(s)),
                            Progress::Optimizing {
                                current: s,
                                index: i,
                                total: t,
                            } => (3, i as i32, t as i32, Some(s)),
                            Progress::Building {
                                current: s,
                                index: i,
                                total: t,
                            } => (4, i as i32, t as i32, Some(s)),
                            Progress::Done => (5, 0, 0, None),
                        };

                        let _ = env.with_local_frame(16, |env| {
                            let j_str_obj: JObject = cur_str
                                .and_then(|s| env.new_string(s).ok())
                                .map(|s| s.into())
                                .unwrap_or_else(|| JObject::null());

                            let _ = env.call_method(
                                &global_prog_cb,
                                jni_str!("onProgress"),
                                jni_sig!("(IIILjava/lang/String;)V"),
                                &[
                                    JValue::Int(state),
                                    JValue::Int(current),
                                    JValue::Int(total),
                                    JValue::Object(&j_str_obj),
                                ],
                            );
                            Ok::<(), jni::errors::Error>(())
                        });
                    }
                    Ok(())
                });
            });
        }

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            process_zip(input, &options, prog_tx, &log_tx, &cache_file)
        }));

        let bytes = match result {
            Ok(Ok(bytes)) => bytes,

            Ok(Err(e)) => {
                let _ = env.throw_new(
                    JNIString::from("java/io/IOException"),
                    JNIString::from(format!("Zip processing failed: {}", e)),
                );
                return Err(Error::JavaException);
            }

            Err(panic) => {
                let msg = if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else {
                    "Unknown Rust panic in process_zip".to_string()
                };

                let _ = env.throw_new(
                    JNIString::from("java/io/IOException"),
                    JNIString::from(format!("Rust panic during zip processing: {}", msg)),
                );

                return Err(Error::JavaException);
            }
        };

        let output: JByteArray = env.byte_array_from_slice(&bytes)?;
        Ok(output.into_raw())
    });

    outcome.resolve::<jni::errors::ThrowRuntimeExAndDefault>()
}
