use crate::lib::error::DfxResult;
use indicatif::ProgressBar;
use notify::{watcher, RecursiveMode, Watcher};
use std::borrow::Borrow;
use std::path::Path;
use std::sync::mpsc::{channel, Sender};
use std::sync::Arc;
use std::thread;
use std::time::Duration;

type BinaryCommandFn = dyn Fn(&str) -> DfxResult<std::process::Command>;

pub fn watch_file_and_spin(
    bar: Arc<ProgressBar>,
    binary_command: Arc<Fn(&str) -> DfxResult<std::process::Command> + Send + Sync>,
    file_path: &Path,
    output_root: &Path,
) -> DfxResult<Sender<()>>
{
    let binary_command_arc = Arc::clone(&binary_command);
    let (tx, rx) = channel();
    let (sender, receiver) = channel();

    build_file_and_spin(Arc::clone(&bar), binary_command_arc.as_ref(), file_path, output_root)?;

    // There's a better way to do this, e.g. with a single thread watching all files, but this
    // works great for a few files.
    let mut watcher = watcher(tx, Duration::from_secs(2))?;
    watcher.watch(file_path, RecursiveMode::NonRecursive)?;

    // Make actual clones of values to move them in the thread.
    let file_path: Box<Path> = Box::from(file_path);
    let output_root: Box<Path> = Box::from(output_root);
    let binary_command_arc = Arc::clone(&binary_command_arc);
    thread::spawn(move || {
        let fp = file_path.borrow();
        let out = output_root.borrow();

        loop {
            if receiver.try_recv().is_ok() {
                break;
            }

            if rx.recv_timeout(Duration::from_millis(80)).is_ok() {
                let arc = Arc::clone(&binary_command_arc);
                build_file_and_spin(Arc::clone(&bar), arc.as_ref(), &fp, &out).unwrap();
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

pub fn build_file_and_spin<'a>(
    bar: Arc<ProgressBar>,
    binary_command: &'a BinaryCommandFn,
    file_path: &'a Path,
    output_root: &'a Path,
) -> DfxResult {
    let (tx, rx) = channel();

    let b = Arc::clone(&bar);
    let t = thread::spawn(move || loop {
        match rx.recv_timeout(Duration::from_millis(80)) {
            Ok(()) => break,
            _ => {}
        }
        b.inc(1);
    });

    // Build.
    bar.set_message(format!("{} - Building...", file_path.display()).as_str());
    let result = build_file(binary_command, file_path, output_root);

    // Indicate to the thread that we're done.
    #[allow(unused_must_use)]
    {
        tx.send(());
        t.join();
    }

    if let Err(err) = &result {
        bar.finish_with_message(format!("{} ERROR: {:?}", file_path.display(), err).as_str());
    } else {
        bar.finish_with_message(format!("{} Done", file_path.display()).as_str());
    }
    result
}

pub fn build_file<'a>(
    binary_command: &'a BinaryCommandFn,
    file_path: &'a Path,
    output_root: &'a Path,
) -> DfxResult {
    let output_wasm_path = output_root.with_extension("wasm");
    let output_idl_path = output_root.with_extension("did");
    let output_js_path = output_root.with_extension("js");

    std::fs::create_dir_all(output_wasm_path.parent().unwrap())?;

    if let Some(ext) = file_path.extension() {
        if ext == "wat" {
            let wat = std::fs::read(file_path)?;
            let wasm = wabt::wat2wasm(wat)?;

            std::fs::write(output_wasm_path, wasm)?;
        } else {
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
        }
    }

    Ok(())
}
