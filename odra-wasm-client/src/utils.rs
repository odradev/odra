use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub struct BalanceFormatter {
    amount_str: String,
    decimals: usize
}

impl BalanceFormatter {
    pub fn new<T: ToString>(amount: &T, decimals: usize) -> Self {
        BalanceFormatter {
            amount_str: amount.to_string(),
            decimals
        }
    }
}

#[wasm_bindgen]
impl BalanceFormatter {
    #[wasm_bindgen(js_name = "fmt")]
    pub fn fmt(&self) -> String {
        let s = self.amount_str.to_string();
        let len = s.len();
        if len <= self.decimals {
            format!("0.{:0>9}", s)
        } else {
            let (int_part, frac_part) = s.split_at(len - 9);
            format!("{}.{}", int_part, frac_part)
        }
    }

    #[wasm_bindgen(js_name = "fmtWithPrecision")]
    pub fn fmt_with_precision(&self, precision: usize) -> String {
        let cspr_str = self.fmt();
        if let Some(dot_pos) = cspr_str.find('.') {
            let end_pos = (dot_pos + 1 + precision).min(cspr_str.len());
            let result = cspr_str[..end_pos].to_string();
            if result.ends_with('.') {
                result.trim_end_matches('.').to_string()
            } else {
                result
            }
        } else {
            cspr_str
        }
    }
}
