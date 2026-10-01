#[cfg(not(target_os = "android"))]
fn main() -> eframe::Result<()> {
    main::run_desktop()
}

#[cfg(target_os = "android")]
fn main() {}
