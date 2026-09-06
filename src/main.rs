use cosmic::app::Settings;
use rooney::app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let settings = Settings::default().size(cosmic::iced::Size::new(2400.0, 2400.0));
    cosmic::app::run::<App>(settings, ())?;
    Ok(())
}
