use std::sync::{Arc, Mutex};

use super::helpers;
use super::{
    animations::Animation, impl_pystimulus_for_wrapper, PyStimulus, Stimulus, StimulusParamValue, StimulusParams,
};
use crate::visual::geometry::Transformation2D;
use crate::visual::geometry::{Anchor, Size};
use cosmic_text::Attrs as ComsicAttrs;
use cosmic_text::Buffer as CosmicBuffer;
use cosmic_text::Family as CosmicFamily;
use cosmic_text::FontSystem as CosmicFontSystem;
use cosmic_text::Metrics as CosmicMetrics;
use cosmic_text::Stretch as CosmicStretch;
use cosmic_text::Style as CosmicStyle;
use cosmic_text::Weight as CosmicWeight;

use psydk_proc::{FromPyStr, StimulusParams};
use strum::EnumString;
use uuid::Uuid;

use crate::visual::color::IntoLinRgba;
use crate::visual::color::LinRgba;
use crate::visual::window::Frame;
use renderer::affine::Affine;
use renderer::brushes::Brush;
use renderer::colors::RGBA;

#[derive(EnumString, Debug, Clone, Copy, PartialEq, FromPyStr)]
#[strum(serialize_all = "snake_case")]
pub enum FontWeight {
    Thin,
    ExtraLight,
    Light,
    Regular,
    Medium,
    SemiBold,
    Bold,
    ExtraBold,
    Black,
}

#[derive(EnumString, Debug, Clone, Copy, PartialEq, FromPyStr)]
#[strum(serialize_all = "snake_case")]
pub enum TextAlignment {
    Left,
    Center,
    Right,
    Justify,
}

#[derive(Clone, Debug)]
pub struct OwnedCosmicAttrs {
    family: String,
    weight: CosmicWeight,
    stretch: CosmicStretch,
    style: CosmicStyle,
}

#[derive(StimulusParams, Clone, Debug)]
pub struct TextParams {
    pub x: Size,
    pub y: Size,
    pub text: String,
    pub font_size: Size,
    pub fill_color: LinRgba,
    pub alpha: f64,
}

#[derive(Debug)]
pub struct TextStimulus {
    id: uuid::Uuid,
    params: TextParams,
    buffer: CosmicBuffer,
    attrs: OwnedCosmicAttrs,
    alignment: TextAlignment,
    anchor: Anchor,
    font: renderer::font::DynamicFontFace,
    font_manager: Arc<Mutex<CosmicFontSystem>>,
    transform: Transformation2D,
    animations: Vec<Animation>,
    visible: bool,
}

impl TextStimulus {
    pub fn new(
        x: Size,
        y: Size,
        text: &str,
        alignment: TextAlignment,
        anchor: Anchor,
        font_size: Size,
        font_family: &str,
        font_weight: FontWeight,
        fill_color: LinRgba,
        alpha: f64,
        transform: Transformation2D,
        experiment_manager: &crate::experiment::ExperimentManager,
    ) -> Self {
        // Attributes indicate what font to choose
        let attrs = ComsicAttrs::new();
        let attrs = attrs.family(CosmicFamily::Name(font_family));
        let attrs = attrs.weight(font_weight.into());
        let attrs = attrs.stretch(CosmicStretch::Normal);
        let attrs = attrs.style(CosmicStyle::Normal);

        let query = cosmic_text::fontdb::Query {
            families: &[attrs.family],
            weight: attrs.weight,
            stretch: attrs.stretch,
            style: attrs.style,
        };

        let font_manager_clone = experiment_manager.font_manager().clone();

        let renderer_factory = experiment_manager.renderer_factory();
        let mut font_manager = experiment_manager.font_manager().lock().unwrap();

        let cosmic_font_id = font_manager.db().query(&query).unwrap();
        let cosmic_font = font_manager.get_font(cosmic_font_id).unwrap();
        let comic_metrics = CosmicMetrics::new(10.0, 10.0);
        // let comic_metrics = CosmicMetrics::new(14.0, 20.0);

        let face_info = font_manager.db().face(cosmic_font_id).unwrap();
        let font_data = cosmic_font.data();
        let font_index = face_info.index;

        assert!(attrs.matches(&face_info));

        let font = renderer_factory.create_font_face(font_data, font_index);
        let mut cosmic_buffer = CosmicBuffer::new(&mut font_manager, comic_metrics);

        Self {
            id: Uuid::new_v4(),
            params: TextParams {
                x,
                y,
                text: text.to_string(),
                font_size,
                fill_color,
                alpha,
            },
            buffer: cosmic_buffer,
            attrs: attrs.into(),
            font,
            alignment,
            anchor,
            font_manager: font_manager_clone,
            transform,
            animations: Vec::new(),
            visible: true,
        }
    }
}

#[derive(Debug, Clone)]
#[pyclass(name = "TextStimulus", extends=PyStimulus)]
pub struct PyTextStimulus();

#[pymethods]
impl PyTextStimulus {
    #[new]
    #[pyo3(signature = (
        text,
        font_size,
        font_family = "Noto Sans",
        font_weight = FontWeight::Regular,
        alignment = TextAlignment::Center,
        alpha = 1.0,
        anchor = Anchor::Center,
        x = IntoSize(Size::Pixels(0.0)),
        y = IntoSize(Size::Pixels(0.0)),
        fill_color = IntoLinRgba::new(0.0, 0.0, 0.0, 1.0),
        transform = Transformation2D::Identity()
    ))]
    fn __new__(
        py: Python,
        text: &str,
        font_size: IntoSize,
        font_family: &str,
        font_weight: FontWeight,
        alignment: TextAlignment,
        alpha: f64,
        anchor: Anchor,
        x: IntoSize,
        y: IntoSize,
        fill_color: IntoLinRgba,
        transform: Transformation2D,
    ) -> (Self, PyStimulus) {
        let experiment_manager = helpers::get_experiment_manager(py).unwrap();
        (
            Self(),
            PyStimulus::new(TextStimulus::new(
                x.into(),
                y.into(),
                text,
                alignment,
                anchor,
                font_size.into(),
                font_family,
                font_weight,
                fill_color.into(),
                alpha,
                transform,
                &experiment_manager,
            )),
        )
    }
}

impl_pystimulus_for_wrapper!(PyTextStimulus, TextStimulus);

impl Stimulus for TextStimulus {
    fn uuid(&self) -> Uuid {
        self.id
    }

    fn draw(&mut self, frame: &mut Frame) {
        if !self.visible {
            return;
        }

        let window = frame.window();
        let window_state = window.lock_state();
        let window_size = window_state.size;
        let screen_props = window_state.physical_screen;
        let mut font_manager = self.font_manager.lock().unwrap();

        // convert physical units to pixels
        let pos_x = self.params.x.eval(window_size, screen_props) as f64;
        let pos_y = self.params.y.eval(window_size, screen_props) as f64;
        let font_size = self.params.font_size.eval(window_size, screen_props) as f64;

        let trans_mat = self.transform.eval(window_size, screen_props);

        let fill_color: RGBA = self.params.fill_color.into();

        // Set a size for the text buffer, in pixels
        self.buffer.set_size(&mut font_manager, None, None);

        self.buffer.set_metrics(
            &mut font_manager,
            CosmicMetrics::new(font_size as f32, font_size as f32),
        );

        let attrs = (&self.attrs).into();

        // Add some text!
        self.buffer
            .set_text(&mut font_manager, &self.params.text, attrs, cosmic_text::Shaping::Basic);

        // Perform shaping
        self.buffer.shape_until_scroll(&mut font_manager, true);

        // get the width and height of the text
        let (bb_width, bb_height) = measure(&self.buffer);
        // let (bb_width, bb_height) = (bb_width as f64, bb_height as f64);

        // depending on the achoring, we need to adjust the position
        let (new_x, new_y) = self
            .anchor
            .to_top_left(pos_x as f32, pos_y as f32, bb_width, bb_height / 2.0);

        let mut glyphs = vec![];

        for run in self.buffer.layout_runs() {
            for glyph in run.glyphs {
                let glyph = renderer::font::Glyph {
                    id: glyph.glyph_id,
                    position: (glyph.x as f32, glyph.y as f32).into(),
                };
                glyphs.push(glyph);
            }
        }

        let brush = Brush::Solid(fill_color);

        frame.scene_mut().draw_glyphs(
            (new_x, -new_y).into(),
            &glyphs,
            &self.font,
            font_size as f32,
            brush,
            Some(self.params.alpha as f32),
            None,
            None,
        );
    }

    fn set_visible(&mut self, visible: bool) {
        self.visible = visible;
    }

    fn visible(&self) -> bool {
        self.visible
    }

    fn animations(&mut self) -> &mut Vec<Animation> {
        &mut self.animations
    }

    fn add_animation(&mut self, animation: Animation) {
        self.animations.push(animation);
    }

    fn set_transformation(&mut self, transformation: crate::visual::geometry::Transformation2D) {
        self.transform = transformation;
    }

    fn transformation(&self) -> crate::visual::geometry::Transformation2D {
        self.transform.clone()
    }

    fn get_param(&self, name: &str) -> Option<StimulusParamValue> {
        self.params.get_param(name)
    }

    fn set_param(&mut self, name: &str, value: StimulusParamValue) {
        self.params.set_param(name, value)
    }
}

// convert FontWeight to CosmicWeight
impl From<FontWeight> for CosmicWeight {
    fn from(weight: FontWeight) -> Self {
        match weight {
            FontWeight::Thin => CosmicWeight::THIN,
            FontWeight::ExtraLight => CosmicWeight::EXTRA_LIGHT,
            FontWeight::Light => CosmicWeight::LIGHT,
            FontWeight::Regular => CosmicWeight::NORMAL,
            FontWeight::Medium => CosmicWeight::MEDIUM,
            FontWeight::SemiBold => CosmicWeight::SEMIBOLD,
            FontWeight::Bold => CosmicWeight::BOLD,
            FontWeight::ExtraBold => CosmicWeight::EXTRA_BOLD,
            FontWeight::Black => CosmicWeight::BLACK,
        }
    }
}

// convert OwnedCosmicAttrs to CosmicAttrs
impl From<&OwnedCosmicAttrs> for ComsicAttrs<'_> {
    fn from(attrs: &OwnedCosmicAttrs) -> Self {
        let mut cosmic_attrs = ComsicAttrs::new();
        cosmic_attrs.family(CosmicFamily::Name(&attrs.family));
        cosmic_attrs.weight(attrs.weight);
        cosmic_attrs.stretch(attrs.stretch);
        cosmic_attrs.style(attrs.style);
        cosmic_attrs
    }
}

// convert CosmicAttrs to OwnedCosmicAttrs
impl From<ComsicAttrs<'_>> for OwnedCosmicAttrs {
    fn from(attrs: ComsicAttrs) -> Self {
        OwnedCosmicAttrs {
            family: match attrs.family {
                CosmicFamily::Name(family) => family.to_string(),
                CosmicFamily::Serif => "serif".to_string(),
                CosmicFamily::SansSerif => "sans-serif".to_string(),
                CosmicFamily::Monospace => "monospace".to_string(),
                CosmicFamily::Cursive => "cursive".to_string(),
                CosmicFamily::Fantasy => "fantasy".to_string(),
            },
            weight: attrs.weight,
            stretch: attrs.stretch,
            style: attrs.style,
        }
    }
}

fn measure(buffer: &CosmicBuffer) -> (f32, f32) {
    buffer.layout_runs().fold((0.0f32, 0.0f32), |size, run| {
        (size.0.max(run.line_w), size.1 + run.line_height)
    })
}
