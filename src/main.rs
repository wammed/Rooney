use cosmic::app::Settings;
use rooney::app::{App, AppFlags};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    for arg in &args {
        match arg.as_str() {
            "-h" | "--help" => {
                println!("Rooney - Cosmic-Native Lightweight Code & Markdown Editor");
                println!();
                println!("Usage: rooney [OPTIONS] [FILE/DIR]...");
                println!();
                println!("Options:");
                println!("  -h, --help     Print help information");
                println!("  -v, --version  Print version information");
                return Ok(());
            }
            "-v" | "--version" => {
                println!("rooney {}", env!("CARGO_PKG_VERSION"));
                return Ok(());
            }
            _ => {}
        }
    }

    let flags = AppFlags::from_args(args);
    let settings = Settings::default().size(cosmic::iced::Size::new(1600.0, 1600.0));
    cosmic::app::run::<App>(settings, flags)?;
    Ok(())
}
