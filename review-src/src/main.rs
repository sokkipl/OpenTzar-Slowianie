// Release builds are a game: no console window behind it. Command-line
// messages attach to the console they were started from.
#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use retzar::{Result, menu::Screen, paths::Locations};
use std::path::PathBuf;

const HELP: &str = "Excalibur e il Re Artù
Senza argomenti: apre il menu

  --skip-intro           salta il filmato all'avvio
  --intro                riproduce l'intro anche con --scenario, --random, --screen o --capture
  --scenario ID          apre subito uno scenario (elenco: retzar-tools --list)
  --at X,Y[,ZOOM]        con --scenario: camera sulla casella X,Y
  --random LATO:SEME     apre una mappa casuale (64, 96, 128, 192, 256, 384 o 512)
  --screen NOME          apre una pagina del menu (Credits, Options, Setup, …)
                         o una scheda delle impostazioni (Audio, Gioco, Schermo,
                         Grafica, Tasti)
  --capture FILE.png     salva uno screenshot dopo l'avvio ed esce
  --run SECONDI          con --capture: la partita va avanti per SECONDI
                         anche senza il fuoco, prima dello screenshot

  --source CARTELLA      installazione originale (oppure TZAR_DATA_ROOT)
  --root CARTELLA        impostazioni e salvataggi (oppure RETZAR_HOME)";

fn main() {
    #[cfg(all(windows, not(debug_assertions)))]
    attach_parent_console();
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--ot-check-assets") {
        if let Err(error) = retzar::runtime_art::check_installation() {
            eprintln!("[ZASOBY] {error}");
            if let Ok(exe) = std::env::current_exe() {
                let _ = std::fs::write(exe.with_file_name("runtime-art-v1.error"), error.to_string());
            }
            std::process::exit(1);
        }
        return;
    }
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("ReTzar {} — {HELP}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let locations = Locations::resolve(&args);
    install_crash_log(locations.as_ref().ok().map(|l| l.root.clone()));
    if let Err(error) = locations.and_then(|mut l| {
        let source = retzar::original::require_direct(&l.root, &l.source)?;
        l.source = source;
        retzar::europe::prepare(&l.source, &l.root)?;
        log_attempt(&l.root, &format!("[START] źródło={}\n", l.source.display()));
        run(&l, &args)
    }) {
        let message = format!("Błąd: {error}");
        eprintln!("{message}");
        if let Ok(dir) = std::env::current_exe().and_then(|p| p.parent()
            .map(std::path::Path::to_path_buf)
            .ok_or_else(|| std::io::Error::other("Nie znaleziono katalogu EXE"))) {
            log_attempt(&dir, &format!("{message}\n"));
        }
        fatal_dialog(&format!("OpenTzar nie może kontynuować:\n\n{error}"));
        std::process::exit(1);
    }
}

fn value<'a>(args: &'a [String], key: &str) -> Option<&'a str> {
    args.windows(2).find(|a| a[0] == key).map(|a| a[1].as_str())
}

/// `--at X,Y[,ZOOM]`: the camera centred on cell (X, Y) at ZOOM× (2 by default).
fn at(text: &str) -> Result<[f32; 3]> {
    let v: Vec<f32> = text
        .split(',')
        .map(|s| s.trim().parse::<f32>())
        .collect::<std::result::Result<_, _>>()
        .map_err(|_| "Usa --at X,Y[,ZOOM], per esempio --at 120,80,2")?;
    let zoom = v.get(2).copied().unwrap_or(2.).clamp(0.25, 8.);
    match v[..] {
        [x, y, ..] => Ok([x * 32. + 16., -(y * 32. + 16.), 1. / zoom]),
        _ => Err("Usa --at X,Y[,ZOOM], per esempio --at 120,80,2".into()),
    }
}

fn run(locations: &Locations, args: &[String]) -> Result<()> {
    let has = |flag: &str| args.iter().any(|a| a == flag);
    let start = if let Some(id) = value(args, "--scenario") {
        Some(frontend::Start::Scenario(id.into()))
    } else if let Some(r) = value(args, "--random") {
        let (side, seed) = r
            .split_once(':')
            .and_then(|(a, b)| Some((a.parse().ok()?, b.parse().ok()?)))
            .ok_or("Usa --random LATO:SEME, per esempio 128:7")?;
        Some(frontend::Start::Random { side, seed })
    } else if let Some(name) = value(args, "--screen") {
        if let Some(tab) = retzar::menu::SettingsTab::named(name) {
            Some(frontend::Start::Settings(tab))
        } else {
            let screen = Screen::ALL
                .into_iter()
                .find(|s| format!("{s:?}").eq_ignore_ascii_case(name))
                .ok_or_else(|| format!("Pagina sconosciuta: {name}"))?;
            Some(frontend::Start::Screen(screen))
        }
    } else {
        None
    };
    let direct = start.is_some() || has("--capture");
    frontend::run(
        locations,
        frontend::Launch {
            capture: value(args, "--capture").map(PathBuf::from),
            run: value(args, "--run")
                .map(|s| {
                    s.parse::<f32>()
                        .ok()
                        .filter(|v| v.is_finite() && *v >= 0.)
                        .map(std::time::Duration::from_secs_f32)
                        .ok_or_else(|| format!("--run: secondi non validi: {s}"))
                })
                .transpose()?,
            start,
            camera: value(args, "--at").map(at).transpose()?,
            intro: has("--intro") || (!has("--skip-intro") && !direct),
        },
    )
}

/// Panics end in `crash.log` next to the settings, with a readable message.
fn log_attempt(root: &std::path::Path, message: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new().create(true).append(true)
        .open(root.join("OpenTzar-ostatnia-proba.txt")) {
        let _ = f.write_all(message.as_bytes());
    }
}

fn install_crash_log(root: Option<PathBuf>) {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        // A command-line listing cut short by its reader (`| head`) is not a
        // crash: stop quietly, as console tools do.
        let message = info
            .payload()
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| info.payload().downcast_ref::<&str>().copied())
            .unwrap_or("");
        if message.starts_with("failed printing to stdout") {
            std::process::exit(0);
        }
        default(info);
        let text = format!(
            "OpenTzar {} — błąd wewnętrzny\n{info}\n\n{}",
            env!("CARGO_PKG_VERSION"),
            std::backtrace::Backtrace::force_capture()
        );
        if let Some(root) = &root {
            let _ = std::fs::write(root.join("crash.log"), &text);
            log_attempt(root, &format!("[CRASH] {info}\n"));
        }
        fatal_dialog(&retzar::lang::fill(
            "OpenTzar zakończył się wskutek błędu.\n\n{}\n\nSzczegóły w crash.log.",
            &[&info],
        ));
    }));
}

#[cfg(all(windows, not(debug_assertions)))]
fn attach_parent_console() {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn AttachConsole(process: u32) -> i32;
    }
    // ATTACH_PARENT_PROCESS: print CLI output in the console that started us.
    unsafe {
        AttachConsole(u32::MAX);
    }
}

/// Errors are visible even without a console (release builds, Explorer).
fn fatal_dialog(text: &str) {
    #[cfg(not(windows))]
    retzar::picker::error(text);
    #[cfg(all(windows, not(debug_assertions)))]
    {
        #[link(name = "user32")]
        unsafe extern "system" {
            fn MessageBoxW(
                window: *mut std::ffi::c_void,
                text: *const u16,
                caption: *const u16,
                kind: u32,
            ) -> i32;
        }
        let wide = |s: &str| s.encode_utf16().chain([0]).collect::<Vec<u16>>();
        let (text, caption) = (wide(text), wide("ReTzar"));
        // MB_ICONERROR
        unsafe {
            MessageBoxW(std::ptr::null_mut(), text.as_ptr(), caption.as_ptr(), 0x10);
        }
    }
    #[cfg(not(all(windows, not(debug_assertions))))]
    let _ = text;
}

mod frontend;
