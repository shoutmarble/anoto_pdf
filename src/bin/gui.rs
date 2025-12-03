use iced::widget::{button, column, container, row, scrollable, slider, text, vertical_space};
use iced::widget::image::{self, Image};
use iced::{Element, Length, Task, ContentFit, Border, Color, Shadow};
use iced_aw::spinner::Spinner;
use anoto_pdf::pdf_dotpaper::gen_pdf::{PdfConfig, gen_pdf_from_matrix_data};
use anoto_pdf::anoto_matrix::generate_matrix_only;
use anoto_pdf::make_plots::draw_preview_image;
use anoto_pdf::controls::{anoto_control, page_layout_control, section_control};

const JB_MONO_BYTES: &[u8] = include_bytes!("../assets/fonts/ttf/JetBrainsMonoNL-Medium.ttf");

pub fn main() -> iced::Result {
    iced::application("Anoto PDF Generator", Gui::update, Gui::view)
        .window_size((800.0, 600.0))
        .centered()
        .scale_factor(|s| s.ui_scale as f64)
        .font(JB_MONO_BYTES)
        .run()
}

struct Gui {
    config: PdfConfig,
    height: usize,
    width: usize,
    sect_u: i32,
    sect_v: i32,
    status_message: String,
    sect_u_str: String,
    sect_v_str: String,
    ui_scale: f32,
    generated_image_handle: Option<image::Handle>,
    control_state: anoto_control::State,
    page_layout_state: page_layout_control::State,
    is_generating: bool,
}

#[derive(Debug, Clone)]
enum Message {
    UiScaleChanged(f32),
    DpiChanged(f32),
    DotSizeChanged(f32),
    OffsetChanged(f32),
    SpacingChanged(f32),
    HeightChanged(usize),
    WidthChanged(usize),
    AutodetectChanged(bool),
    SectUChanged(i32),
    SectVChanged(i32),
    GeneratePressed,
    GenerationFinished(Result<image::Handle, String>),
    ToggleUpPicker(bool),
    ToggleDownPicker(bool),
    ToggleLeftPicker(bool),
    ToggleRightPicker(bool),
    ColorUpPicked(Color),
    ColorDownPicked(Color),
    ColorLeftPicked(Color),
    ColorRightPicked(Color),
}

impl Default for Gui {
    fn default() -> Self {
        Self {
            config: PdfConfig::default(),
            height: 9,
            width: 16,
            sect_u: 10,
            sect_v: 10,
            status_message: "Ready".to_string(),
            sect_u_str: "10".to_string(),
            sect_v_str: "10".to_string(),
            ui_scale: 0.6,
            generated_image_handle: None,
            control_state: anoto_control::State::default(),
            page_layout_state: page_layout_control::State::default(),
            is_generating: false,
        }
    }
}

#[derive(Clone)]
struct GenerationParams {
    height: usize,
    width: usize,
    sect_u: i32,
    sect_v: i32,
    config: PdfConfig,
}

impl Gui {
    fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::UiScaleChanged(val) => self.ui_scale = val,
            Message::DpiChanged(val) => self.config.dpi = val,
            Message::DotSizeChanged(val) => self.config.dot_size = (val * 10.0).round() / 10.0,
            Message::OffsetChanged(val) => self.config.offset_from_origin = (val * 10.0).round() / 10.0,
            Message::SpacingChanged(val) => {
                self.config.grid_spacing = (val * 10.0).round() / 10.0;
                if self.page_layout_state.autodetect {
                    self.recalculate_layout();
                }
            },
            Message::HeightChanged(val) => {
                if !self.page_layout_state.autodetect {
                    self.height = val;
                }
            },
            Message::WidthChanged(val) => {
                if !self.page_layout_state.autodetect {
                    self.width = val;
                }
            },
            Message::AutodetectChanged(val) => {
                self.page_layout_state.autodetect = val;
                if val {
                    self.recalculate_layout();
                }
            },
            Message::SectUChanged(val) => {
                self.sect_u = val;
                self.sect_u_str = val.to_string();
            },
            Message::SectVChanged(val) => {
                self.sect_v = val;
                self.sect_v_str = val.to_string();
            },
            Message::ToggleUpPicker(show) => self.control_state.show_up = show,
            Message::ToggleDownPicker(show) => self.control_state.show_down = show,
            Message::ToggleLeftPicker(show) => self.control_state.show_left = show,
            Message::ToggleRightPicker(show) => self.control_state.show_right = show,
            Message::ColorUpPicked(color) => {
                self.config.color_up = color_to_hex(color);
                self.control_state.show_up = false;
            },
            Message::ColorDownPicked(color) => {
                self.config.color_down = color_to_hex(color);
                self.control_state.show_down = false;
            },
            Message::ColorLeftPicked(color) => {
                self.config.color_left = color_to_hex(color);
                self.control_state.show_left = false;
            },
            Message::ColorRightPicked(color) => {
                self.config.color_right = color_to_hex(color);
                self.control_state.show_right = false;
            },
            Message::GeneratePressed => {
                if !self.is_generating {
                    self.is_generating = true;
                    self.status_message = "Generating PDF...".to_string();
                    let params = GenerationParams {
                        height: self.height,
                        width: self.width,
                        sect_u: self.sect_u,
                        sect_v: self.sect_v,
                        config: self.config.clone(),
                    };
                    return Task::perform(async move {
                        generate_and_save(params).await
                    }, Message::GenerationFinished);
                }
            },
            Message::GenerationFinished(result) => {
                self.is_generating = false;
                match result {
                    Ok(handle) => {
                        self.status_message = "PDF Generated Successfully!".to_string();
                        self.generated_image_handle = Some(handle);
                    }
                    Err(e) => self.status_message = format!("Error: {}", e),
                }
            }
        }
        Task::none()
    }

    fn recalculate_layout(&mut self) {
        let a4_width = 595.276;
        let a4_height = 841.89;
        let margin = 20.0;
        let spacing = self.config.grid_spacing;
        if spacing > 0.0 {
            self.width = ((a4_width - 2.0 * margin) / spacing) as usize + 1;
            self.height = ((a4_height - 2.0 * margin) / spacing) as usize + 1;
        }
    }

    fn view(&self) -> Element<'_, Message> {
        let scale_slider = container(column![
            text(format!("UI Scale: {:.1}", self.ui_scale)),
            slider(0.5..=3.0, self.ui_scale, Message::UiScaleChanged).step(0.1)
        ].spacing(10))
        .padding(10)
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        });

        let dpi_slider = container(column![
            text(format!("PDF DPI: {:.0}", self.config.dpi)),
            slider(300.0..=1200.0, self.config.dpi, Message::DpiChanged).step(10.0)
        ].spacing(10))
        .padding(10)
        .style(|_theme| container::Style {
            border: Border {
                color: Color::from_rgb(0.5, 0.5, 0.5),
                width: 1.0,
                radius: 5.0.into(),
            },
            ..container::Style::default()
        });

        let anoto_ctrl = anoto_control::anoto_control(
            &self.control_state,
            self.config.dot_size,
            self.config.grid_spacing,
            self.config.offset_from_origin,
            hex_to_color(&self.config.color_up),
            hex_to_color(&self.config.color_down),
            hex_to_color(&self.config.color_left),
            hex_to_color(&self.config.color_right),
            Message::ToggleUpPicker,
            Message::ToggleDownPicker,
            Message::ToggleLeftPicker,
            Message::ToggleRightPicker,
            Message::ColorUpPicked,
            Message::ColorDownPicked,
            Message::ColorLeftPicked,
            Message::ColorRightPicked,
            Message::DotSizeChanged,
            Message::SpacingChanged,
            Message::OffsetChanged,
        );

        let page_layout = page_layout_control::page_layout_control(
            &self.page_layout_state,
            self.width,
            self.height,
            Message::WidthChanged,
            Message::HeightChanged,
            Message::AutodetectChanged,
        );

        let section_ctrl = section_control::section_control(
            self.sect_u,
            self.sect_v,
            Message::SectUChanged,
            Message::SectVChanged,
        );

        let matrix_inputs = column![
            text("Matrix Settings:"),
            page_layout,
            section_ctrl,
        ].spacing(10);

        let generate_btn = if self.is_generating {
            row![
                button("Generating...").width(Length::Fill),
                Spinner::new().width(Length::Fixed(20.0)).height(Length::Fixed(20.0)),
            ].spacing(10)
        } else {
            row![
                button("Generate PDF").on_press(Message::GeneratePressed).width(Length::Fill)
            ]
        };

        let controls = column![
            text("Anoto PDF Generator").size(30),
            vertical_space().height(20),
            scale_slider,
            vertical_space().height(20),
            dpi_slider,
            vertical_space().height(20),
            anoto_ctrl,
            vertical_space().height(20),
            matrix_inputs,
            vertical_space().height(20),
            generate_btn,
            vertical_space().height(10),
            text(&self.status_message),
        ]
        .spacing(10)
        .padding(20)
        .width(Length::Fixed(460.0));

        let image_preview = if let Some(handle) = &self.generated_image_handle {
             container(
                container(
                    Image::new(handle.clone())
                        .content_fit(ContentFit::Contain)
                        .width(Length::Fill)
                        .height(Length::Fill)
                )
                .padding(5)
                .style(|_theme| container::Style {
                    border: Border {
                        color: Color::from_rgb(0.2, 0.2, 0.2),
                        width: 5.0,
                        radius: 10.0.into(),
                    },
                    background: Some(Color::from_rgb(0.95, 0.95, 0.95).into()),
                    shadow: Shadow {
                        color: Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                        offset: iced::Vector::new(5.0, 5.0),
                        blur_radius: 10.0,
                    },
                    ..container::Style::default()
                })
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
        } else {
            container(text("No image generated yet").size(20))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x(Length::Fill)
                .center_y(Length::Fill)
        };

        row![
            image_preview,
            scrollable(controls).width(Length::Fixed(480.0))
        ]
        .into()
    }
}

async fn generate_and_save(params: GenerationParams) -> Result<image::Handle, String> {
    // This is a blocking operation, but we run it in an async block.
    // In a real async runtime, we should use spawn_blocking.
    // Since we don't have easy access to spawn_blocking without adding tokio dependency explicitly,
    // we will just run it here. It might block the UI thread if the executor is single threaded.
    // However, for the purpose of this task, we are using Task::perform.
    
    let result = (|| -> Result<image::Handle, Box<dyn std::error::Error>> {
        let bitmatrix = generate_matrix_only(params.height, params.width, params.sect_u, params.sect_v)?;
        let base_filename = format!("GUI_G__{}__{}__{}__{}", params.height, params.width, params.sect_u, params.sect_v);
        
        // Generate PDF
        gen_pdf_from_matrix_data(&bitmatrix, &format!("{}.pdf", base_filename), &params.config)?;

        // Generate PNG
        if !std::path::Path::new("output").exists() {
            std::fs::create_dir("output")?;
        }
        let bitmatrix_i8 = bitmatrix.mapv(|x| x as i8);
        let png_path = format!("output/{}__X.png", base_filename);
        draw_preview_image(&bitmatrix_i8, &params.config, &png_path)?;

        // Load image bytes to force refresh
        let bytes = std::fs::read(&png_path)?;
        Ok(image::Handle::from_bytes(bytes))
    })();

    result.map_err(|e| e.to_string())
}

fn color_to_hex(color: Color) -> String {
    let r = (color.r * 255.0).round() as u8;
    let g = (color.g * 255.0).round() as u8;
    let b = (color.b * 255.0).round() as u8;
    format!("#{:02X}{:02X}{:02X}", r, g, b)
}

fn hex_to_color(hex: &str) -> Color {
    if hex.len() == 7 && hex.starts_with('#') {
        let r = u8::from_str_radix(&hex[1..3], 16).unwrap_or(0);
        let g = u8::from_str_radix(&hex[3..5], 16).unwrap_or(0);
        let b = u8::from_str_radix(&hex[5..7], 16).unwrap_or(0);
        Color::from_rgb8(r, g, b)
    } else {
        Color::BLACK
    }
}


