use crate::lib::error::{DfxError, DfxResult};
use notify::{watcher, RecursiveMode, Watcher};
use std::borrow::Borrow;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Sender};
use std::thread;
use std::time::Duration;

type BinaryCommandFn = dyn Fn(&str) -> DfxResult<std::process::Command>;

pub fn watch_file(
    binary_command: Box<dyn Fn(&str) -> DfxResult<std::process::Command> + Send + Sync>,
    file_path: &Path,
    output_root: &Path,
    on_start: Box<dyn Fn() -> () + Send + Sync>,
    on_done: Box<dyn Fn(PathBuf) -> () + Send + Sync>,
    on_error: Box<dyn Fn() -> () + Send + Sync>,
) -> DfxResult<Sender<()>> {
    let (tx, rx) = channel();
    let (sender, receiver) = channel();

    // There's a better way to do this, e.g. with a single thread watching all files, but this
    // works great for a few files.
    let mut watcher = watcher(tx, Duration::from_secs(2))?;
    watcher.watch(file_path, RecursiveMode::NonRecursive)?;

    // Make actual clones of values to move them in the thread.
    let file_path: Box<Path> = Box::from(file_path);
    let output_root: Box<Path> = Box::from(output_root);
    thread::spawn(move || {
        let fp = file_path.borrow();
        let out = output_root.borrow();

        on_start();
        let pb = build_file(&binary_command, &fp, &out).unwrap();
        on_done(pb);

        loop {
            if receiver.try_recv().is_ok() {
                break;
            }

            if rx.recv_timeout(Duration::from_millis(80)).is_ok() {
                on_start();
                match build_file(&binary_command, &fp, &out) {
                    Ok(pb) => on_done(pb),
                    Err(_) => on_error(),
                };
            }
        }

        // Ignore result from unwatch. Nothing we can do.
        #[allow(unused_must_use)]
        {
            watcher.unwatch(fp);
        }
    });

    Ok(sender)
}

pub fn build_file<'a>(
    binary_command: &'a BinaryCommandFn,
    file_path: &'a Path,
    output_root: &'a Path,
) -> DfxResult<PathBuf> {
    let output_wasm_path = output_root.with_extension("wasm");
    let output_idl_path = output_root.with_extension("did");
    let output_js_path = output_root.with_extension("js");

    std::fs::create_dir_all(output_wasm_path.parent().unwrap())?;

    match file_path.extension().and_then(OsStr::to_str) {
        Some("wat") => {
            let wat = std::fs::read(file_path)?;
            let wasm = wabt::wat2wasm(wat)?;

            std::fs::write(&output_wasm_path, wasm)?;

            Ok(())
        }
        Some("as") => {
            binary_command("asc")?
                .arg(&file_path)
                .arg("-o")
                .arg(&output_wasm_path)
                .output()?;
            binary_command("asc")?
                .arg("--idl")
                .arg(&file_path)
                .arg("-o")
                .arg(&output_idl_path)
                .output()?;
            binary_command("didc")?
                .arg("--js")
                .arg(&output_idl_path)
                .arg("-o")
                .arg(&output_js_path)
                .output()?;

            Ok(())
        }
        Some(ext) => Err(DfxError::Unknown(format!(
            r#"Extension unsupported "{}"."#,
            ext
        ))),
        None => Err(DfxError::Unknown(format!(r#"Extension unsupported ""."#))),
    }?;

    thread::sleep(Duration::from_millis(400));

    Ok(output_wasm_path)
}
