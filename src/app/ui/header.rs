use crate::app::message::{ActiveHeaderMenu, Message};
use crate::app::App;
use crate::editor::SplitLayout;
use crate::ui::file_tree_view::FileTreeMessage;
use cosmic::iced::widget::{column, row};
use cosmic::iced::{Alignment, Length};
use cosmic::prelude::*;
use cosmic::widget::{button, container, text, Space};

const ROONEY_ICON_BYTES: &[u8] = include_bytes!("../../../images/Rooney-matte-icon.svg");

impl App {
    pub(crate) fn render_header_start(&self) -> Vec<Element<'_, Message>> {
        let icon_handle = cosmic::iced::widget::svg::Handle::from_memory(ROONEY_ICON_BYTES);
        let app_icon: Element<'_, Message> = container(
            cosmic::widget::svg(icon_handle)
                .width(Length::Fixed(22.0))
                .height(Length::Fixed(22.0)),
        )
        .padding([0, 5])
        .align_y(Alignment::Center)
        .into();

        vec![
            app_icon,
            button::text(if self.active_header_menu == Some(ActiveHeaderMenu::File) {
                " 󰈔 File ▴"
            } else {
                " 󰈔 File ▾"
            })
            .on_press(Message::ToggleHeaderMenu(ActiveHeaderMenu::File))
            .into(),
            button::text(if self.active_header_menu == Some(ActiveHeaderMenu::Edit) {
                " 󰧑 Edit ▴"
            } else {
                " 󰧑 Edit ▾"
            })
            .on_press(Message::ToggleHeaderMenu(ActiveHeaderMenu::Edit))
            .into(),
            button::text(if self.active_header_menu == Some(ActiveHeaderMenu::View) {
                " 󰈈 View ▴"
            } else {
                " 󰈈 View ▾"
            })
            .on_press(Message::ToggleHeaderMenu(ActiveHeaderMenu::View))
            .into(),
            button::text(if self.active_header_menu == Some(ActiveHeaderMenu::Ai) {
                " 󰚩 AI ▴"
            } else {
                " 󰚩 AI ▾"
            })
            .on_press(Message::ToggleHeaderMenu(ActiveHeaderMenu::Ai))
            .into(),
            button::text(" 󰒓 Aesthetics ")
                .on_press(Message::ToggleSettings)
                .into(),
        ]
    }

    pub(crate) fn render_title_bar(&self) -> Element<'_, Message> {
        let mut hb = cosmic::widget::header_bar()
            .title(&self.core.window.header_title)
            .focused(true)
            .maximized(self.core.window.is_maximized)
            .on_drag(Message::DragWindow)
            .on_double_click(Message::MaximizeWindow)
            .on_close(Message::CloseWindow)
            .on_maximize(Message::MaximizeWindow)
            .on_minimize(Message::MinimizeWindow);

        for elem in self.render_header_start() {
            hb = hb.start(elem);
        }

        for elem in self.render_header_end() {
            hb = hb.end(elem);
        }

        let bg_color = self.theme.title_bar_with_alpha();
        container(hb)
            .class(cosmic::theme::Container::Custom(Box::new(move |_| {
                container::Style {
                    background: Some(bg_color.into()),
                    ..Default::default()
                }
            })))
            .width(Length::Fill)
            .into()
    }

    pub(crate) fn render_header_end(&self) -> Vec<Element<'_, Message>> {
        vec![]
    }

    pub(crate) fn render_header_menu_overlay<'a>(
        &'a self,
        base_view: Element<'a, Message>,
    ) -> Element<'a, Message> {
        let Some(menu) = self.active_header_menu else {
            return base_view;
        };

        let theme = &self.theme;
        let menu_w = 240.0;
        let menu_y = 46.0;
        let menu_x = match menu {
            ActiveHeaderMenu::File => 44.0,
            ActiveHeaderMenu::Edit => 128.0,
            ActiveHeaderMenu::View => 212.0,
            ActiveHeaderMenu::Ai => 300.0,
        };

        let make_item =
            |icon: &'static str, label: &'static str, shortcut: &'static str, msg: Message| {
                let content = row::with_capacity(3)
                    .spacing(8)
                    .align_y(Alignment::Center)
                    .padding([5, 10])
                    .push(
                        text(format!("{icon}  {label}"))
                            .size(12.5)
                            .class(cosmic::theme::Text::Color(theme.config.fg)),
                    )
                    .push(cosmic::iced::widget::space::horizontal())
                    .push(
                        text(shortcut)
                            .size(11.0)
                            .class(cosmic::theme::Text::Color(theme.config.comment)),
                    );

                button::custom(content)
                    .on_press(msg)
                    .class(cosmic::theme::Button::Transparent)
                    .width(Length::Fill)
            };

        let mut menu_items = column::with_capacity(8).spacing(2).padding(4);

        match menu {
            ActiveHeaderMenu::File => {
                menu_items = menu_items
                    .push(make_item("", "New File", "Ctrl+N", Message::PromptNewFile))
                    .push(make_item("󰈔", "Open File...", "Ctrl+O", Message::OpenFilePrompt))
                    .push(make_item("", "Open Folder...", "Ctrl+Shift+O", Message::OpenFolderPrompt))
                    .push(make_item("󰆓", "Save", "Ctrl+S", Message::SaveFile))
                    .push(make_item("󰆓", "Save As...", "Ctrl+Shift+S", Message::SaveFileAsPrompt))
                    .push(make_item(
                        if self.file_tree.is_visible { "" } else { "" },
                        if self.file_tree.is_visible { "Hide File Tree" } else { "Show File Tree" },
                        "Ctrl+B",
                        Message::FileTreeMsg(FileTreeMessage::ToggleVisibility),
                    ));
            }
            ActiveHeaderMenu::Edit => {
                menu_items = menu_items
                    .push(make_item("󰕌", "Undo", "Ctrl+Z", Message::Undo))
                    .push(make_item("󰑎", "Redo", "Ctrl+Y", Message::Redo))
                    .push(make_item("󰆐", "Cut", "Ctrl+X", Message::Cut))
                    .push(make_item("󰆏", "Copy", "Ctrl+C", Message::Copy))
                    .push(make_item("󰆒", "Paste", "Ctrl+V", Message::Paste))
                    .push(make_item("󰒅", "Select All", "Ctrl+A", Message::SelectAll));
            }
            ActiveHeaderMenu::View => {
                let is_split = self.split_layout == SplitLayout::Split;
                let is_preview = self.current_pane().is_markdown_preview;

                menu_items = menu_items
                    .push(make_item(
                        "",
                        if is_split { "Single Pane" } else { "Split Pane" },
                        "Ctrl+\\",
                        Message::ToggleSplit,
                    ))
                    .push(make_item(
                        if is_preview { "" } else { "" },
                        if is_preview { "Markdown Edit" } else { "Markdown Preview" },
                        "Ctrl+M",
                        Message::ToggleMarkdownPreview,
                    ))
                    .push(make_item("", "Find in File", "Ctrl+F", Message::ToggleSearch));
            }
            ActiveHeaderMenu::Ai => {
                let ai_on = self.ollama.is_enabled;

                menu_items = menu_items
                    .push(make_item(
                        "󰚩",
                        if ai_on { "Disable FIM" } else { "Enable FIM" },
                        "Ctrl+I",
                        Message::ToggleAi,
                    ))
                    .push(make_item(
                        "󰭹",
                        if self.show_ai_chat { "Close Chat Panel" } else { "Open Chat Panel" },
                        "Ctrl+Shift+A",
                        Message::ToggleAiChat,
                    ))
                    .push(make_item("󰚩", "Trigger Suggestion", "Alt+Enter", Message::TriggerAiFim));
            }
        }

        let menu_box = container(menu_items)
            .class(cosmic::theme::Container::Card)
            .padding(4)
            .width(Length::Fixed(menu_w));

        let positioned = row::with_capacity(2)
            .push(Space::new().width(Length::Fixed(menu_x)))
            .push(
                column::with_capacity(2)
                    .push(Space::new().height(Length::Fixed(menu_y)))
                    .push(menu_box),
            )
            .width(Length::Fill)
            .height(Length::Fill);

        let backdrop = button::custom(Space::new().width(Length::Fill).height(Length::Fill))
            .on_press(Message::CloseHeaderMenu)
            .class(cosmic::theme::Button::Transparent);

        cosmic::iced::widget::stack(vec![base_view, backdrop.into(), positioned.into()]).into()
    }
}
