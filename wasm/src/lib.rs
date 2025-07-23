use wasm_bindgen::prelude::*;
use qrcode::{QrCode, render::svg};

#[wasm_bindgen]
pub struct QrGenerator;

#[wasm_bindgen]
impl QrGenerator {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        QrGenerator
    }

    /// Generate a QR code as SVG string from the given text
    #[wasm_bindgen]
    pub fn generate_svg(&self, text: &str, size: u32) -> Result<String, JsValue> {
        let code = QrCode::new(text)
            .map_err(|e| JsValue::from_str(&format!("Failed to generate QR code: {e}")))?;
        
        let svg = code.render::<svg::Color>()
            .min_dimensions(size, size)
            .build();
        
        Ok(svg)
    }
}

impl Default for QrGenerator {
    fn default() -> Self {
        Self::new()
    }
}