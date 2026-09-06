use cosmic::app::Settings;
use rooney::app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default();
    cosmic::app::run::<App>(settings, ())?;
    Ok(())
}
