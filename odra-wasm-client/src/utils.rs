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
        let s = self.amount_str.as_str();
        let len = s.len();
        if self.decimals == 0 {
            return s.to_string();
        }
        if len <= self.decimals {
            return format!("0.{:0>width$}", s, width = self.decimals);
        }
        let split = len - self.decimals;
        let (int_part, frac_part) = s.split_at(split);
        format!("{}.{}", int_part, frac_part)
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
