use cosmic::app::Settings;
use rooney::app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default().size(cosmic::iced::Size::new(1600.0, 1600.0));
    cosmic::app::run::<App>(settings, ())?;
    Ok(())
}
