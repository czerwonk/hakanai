// SPDX-License-Identifier: Apache-2.0

use qrcode::{QrCode, render::svg};
use wasm_bindgen::prelude::*;

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
        if text.len() > 256 {
            return Err(JsValue::from_str("Input text is too long"));
        }

        if size > 250 {
            return Err(JsValue::from_str("Size is too large"));
        }

        let code = QrCode::new(text)
            .map_err(|e| JsValue::from_str(&format!("Failed to generate QR code: {e}")))?;

        let svg = code
            .render::<svg::Color>()
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
