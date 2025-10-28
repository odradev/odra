use gloo_utils::format::JsValueSerdeExt;
use std::ops::Deref;
use wasm_bindgen::prelude::*;

macro_rules! impl_big_int {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
        #[wasm_bindgen(inspectable)]
        pub struct $name(casper_types::$name);

        #[wasm_bindgen]
        impl $name {
            #[wasm_bindgen(constructor)]
            pub fn from_dec_str(value: &str) -> Result<Self, JsError> {
                Ok($name(
                    casper_types::$name::from_dec_str(value)
                        .map_err(|e| JsError::new(&format!("{:?}", e)))?
                ))
            }

            #[wasm_bindgen(js_name = "fromNumber")]
            pub fn from_u32(value: u32) -> Self {
                $name(casper_types::$name::from(value))
            }

            #[wasm_bindgen(js_name = "fromHtmlInput")]
            pub fn from_input(input: web_sys::HtmlInputElement) -> Result<Self, JsError> {
                let value = input.value();
                Self::from_dec_str(value.trim())
            }

            #[wasm_bindgen(js_name = "fromBigInt")]
            pub fn from_js_big_int(value: &js_sys::BigInt) -> Result<Self, JsError> {
                let v = value
                    .to_string(10)
                    .map(|s| s.as_string())
                    .unwrap_or_default()
                    .unwrap_or_default();
                Self::from_dec_str(&v)
            }

            #[wasm_bindgen(js_name = "toString")]
            pub fn to_string_js_alias(&self) -> String {
                self.0.to_string()
            }

            #[wasm_bindgen(js_name = "toJson")]
            pub fn to_json(&self) -> JsValue {
                JsValue::from_serde(self).unwrap_or(JsValue::null())
            }

            #[wasm_bindgen(js_name = "formatter")]
            pub fn formatter(&self, decimals: usize) -> crate::utils::BalanceFormatter {
                crate::utils::BalanceFormatter::new(self, decimals)
            }

            #[wasm_bindgen(js_name = "mul")]
            pub fn mul(&self, other: &Self) -> Self {
                Self(self.0 * other.0)
            }

            #[wasm_bindgen(js_name = "mulBigInt")]
            pub fn mul_bigint(&self, other: &js_sys::BigInt) -> Result<Self, JsError> {
                let other = Self::from_js_big_int(other)?;
                Ok(Self(self.0 * other.0))
            }

            #[wasm_bindgen(js_name = "div")]
            pub fn div(&self, other: &Self) -> Self {
                Self(self.0 / other.0)
            }

            #[wasm_bindgen(js_name = "divBigInt")]
            pub fn div_bigint(&self, other: &js_sys::BigInt) -> Result<Self, JsError> {
                let other = Self::from_js_big_int(other)?;
                Ok(Self(self.0 / other.0))
            }

            #[wasm_bindgen(js_name = "add")]
            pub fn add(&self, other: &Self) -> Self {
                Self(self.0 + other.0)
            }

            #[wasm_bindgen(js_name = "addBigInt")]
            pub fn add_bigint(&self, other: &js_sys::BigInt) -> Result<Self, JsError> {
                let other = Self::from_js_big_int(other)?;
                Ok(Self(self.0 + other.0))
            }

            #[wasm_bindgen(js_name = "sub")]
            pub fn sub(&self, other: &Self) -> Self {
                Self(self.0 - other.0)
            }

            #[wasm_bindgen(js_name = "subBigInt")]
            pub fn sub_bigint(&self, other: &js_sys::BigInt) -> Result<Self, JsError> {
                let other = Self::from_js_big_int(other)?;
                Ok(Self(self.0 - other.0))
            }

            #[wasm_bindgen(js_name = "checkedMul")]
            pub fn checked_mul(&self, other: &Self) -> Option<Self> {
                self.0.checked_mul(other.0).map(Self)
            }

            #[wasm_bindgen(js_name = "checkedAdd")]
            pub fn checked_add(&self, other: &Self) -> Option<Self> {
                self.0.checked_add(other.0).map(Self)
            }

            #[wasm_bindgen(js_name = "checkedSub")]
            pub fn checked_sub(&self, other: &Self) -> Option<Self> {
                self.0.checked_sub(other.0).map(Self)
            }

            #[wasm_bindgen(js_name = "checkedDiv")]
            pub fn checked_div(&self, other: &Self) -> Option<Self> {
                self.0.checked_div(other.0).map(Self)
            }

            #[wasm_bindgen(js_name = "checkedRem")]
            pub fn checked_rem(&self, other: &Self) -> Option<Self> {
                self.0.checked_rem(other.0).map(Self)
            }

            #[wasm_bindgen(js_name = "checkedPow")]
            pub fn checked_pow(&self, exp: u32) -> Option<Self> {
                self.0.checked_pow(*Self::from_u32(exp)).map(Self)
            }

            #[wasm_bindgen(js_name = "toBigInt")]
            pub fn to_big_int(&self) -> js_sys::BigInt {
                JsValue::from(self.0.to_string()).unchecked_into()
            }

            #[wasm_bindgen(js_name = "lt")]
            pub fn lt(&self, other: &Self) -> bool {
                self.0 < other.0
            }

            #[wasm_bindgen(js_name = "le")]
            pub fn le(&self, other: &Self) -> bool {
                self.0 <= other.0
            }

            #[wasm_bindgen(js_name = "gt")]
            pub fn gt(&self, other: &Self) -> bool {
                self.0 > other.0
            }

            #[wasm_bindgen(js_name = "ge")]
            pub fn ge(&self, other: &Self) -> bool {
                self.0 >= other.0
            }

            #[wasm_bindgen(getter)]
            pub fn value(&self) -> String {
                self.to_string()
            }

            #[wasm_bindgen(js_name = "MAX")]
            pub fn max_value() -> Self {
                Self(casper_types::$name::MAX)
            }

            #[wasm_bindgen(js_name = "zero")]
            pub fn zero() -> Self {
                Self(casper_types::$name::zero())
            }
        }

        impl Deref for $name {
            type Target = casper_types::$name;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl From<$name> for casper_types::$name {
            fn from(value: $name) -> Self {
                value.0
            }
        }

        impl From<casper_types::$name> for $name {
            fn from(value: casper_types::$name) -> Self {
                $name(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                $name::from_dec_str(&value).unwrap_or_else(|_| $name(casper_types::$name::zero()))
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                $name::from_dec_str(value).unwrap_or_else(|_| $name(casper_types::$name::zero()))
            }
        }

        impl Default for $name {
            fn default() -> Self {
                $name(casper_types::$name::zero())
            }
        }

        impl core::fmt::Display for $name {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                write!(f, "{:?}", self.0)
            }
        }
    };
}

impl_big_int!(U128);
impl_big_int!(U256);
impl_big_int!(U512);

#[wasm_bindgen]
pub struct OverflowingResultU128 {
    #[wasm_bindgen(readonly)]
    pub result: U128,
    #[wasm_bindgen(readonly)]
    pub overflow: bool
}

#[wasm_bindgen]
pub struct OverflowingResultU256 {
    #[wasm_bindgen(readonly)]
    pub result: U256,
    #[wasm_bindgen(readonly)]
    pub overflow: bool
}

#[wasm_bindgen]
pub struct OverflowingResultU512 {
    #[wasm_bindgen(readonly)]
    pub result: U512,
    #[wasm_bindgen(readonly)]
    pub overflow: bool
}

#[wasm_bindgen]
impl U128 {
    #[wasm_bindgen(js_name = "overflowingMul")]
    pub fn overflowing_mul(&self, other: &Self) -> OverflowingResultU128 {
        let (res, overflow) = self.0.overflowing_mul(other.0);
        OverflowingResultU128 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "overflowingAdd")]
    pub fn overflowing_add(&self, other: &Self) -> OverflowingResultU128 {
        let (res, overflow) = self.0.overflowing_add(other.0);
        OverflowingResultU128 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "overflowingSub")]
    pub fn overflowing_sub(&self, other: &Self) -> OverflowingResultU128 {
        let (res, overflow) = self.0.overflowing_sub(other.0);
        OverflowingResultU128 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "overflowingPow")]
    pub fn overflowing_pow(&self, exp: u32) -> OverflowingResultU128 {
        let (res, overflow) = self.0.overflowing_pow(*Self::from_u32(exp));
        OverflowingResultU128 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "fromU512")]
    pub fn from_u512(value: U512) -> Result<Self, JsError> {
        let max_u128 = (casper_types::U512::one() << 128) - 1;
        if value.0 > max_u128 {
            return Err(JsError::new("Value exceeds U128 maximum"));
        }
        let src = value.0 .0;
        let mut words = [0u64; 2];
        words.copy_from_slice(&src[0..2]);
        Ok(casper_types::U128(words).into())
    }

    #[wasm_bindgen(js_name = "toU512")]
    pub fn to_u512(&self) -> U512 {
        let mut bytes = [0u8; 16];
        self.to_little_endian(&mut bytes);
        casper_types::U512::from_little_endian(&bytes).into()
    }

    #[wasm_bindgen(js_name = "fromU256")]
    pub fn from_u256(value: U256) -> Result<Self, JsError> {
        let max_u128 = (casper_types::U256::one() << 128) - 1;
        if value.0 > max_u128 {
            return Err(JsError::new("Value exceeds U128 maximum"));
        }
        let src = value.0 .0;
        let mut words = [0u64; 2];
        words.copy_from_slice(&src[0..2]);
        Ok(casper_types::U128(words).into())
    }

    #[wasm_bindgen(js_name = "toU256")]
    pub fn to_u256(&self) -> U256 {
        let mut bytes = [0u8; 16];
        self.to_little_endian(&mut bytes);
        casper_types::U256::from_little_endian(&bytes).into()
    }
}

#[wasm_bindgen]
impl U256 {
    #[wasm_bindgen(js_name = "overflowingMul")]
    pub fn overflowing_mul(&self, other: &Self) -> OverflowingResultU256 {
        let (res, overflow) = self.0.overflowing_mul(other.0);
        OverflowingResultU256 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "overflowingAdd")]
    pub fn overflowing_add(&self, other: &Self) -> OverflowingResultU256 {
        let (res, overflow) = self.0.overflowing_add(other.0);
        OverflowingResultU256 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "overflowingSub")]
    pub fn overflowing_sub(&self, other: &Self) -> OverflowingResultU256 {
        let (res, overflow) = self.0.overflowing_sub(other.0);
        OverflowingResultU256 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "overflowingPow")]
    pub fn overflowing_pow(&self, exp: u32) -> OverflowingResultU256 {
        let (res, overflow) = self.0.overflowing_pow(*Self::from_u32(exp));
        OverflowingResultU256 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "fromU512")]
    pub fn from_u512(value: U512) -> Result<Self, JsError> {
        let max_u256 = (casper_types::U512::one() << 256) - 1;
        if value.0 > max_u256 {
            return Err(JsError::new("Value exceeds U256 maximum"));
        }
        let src = value.0 .0;
        let mut words = [0u64; 4];
        words.copy_from_slice(&src[0..4]);
        Ok(U256(casper_types::U256(words)))
    }

    #[wasm_bindgen(js_name = "toU512")]
    pub fn to_u512(&self) -> U512 {
        let mut bytes = [0u8; 32];
        self.to_little_endian(&mut bytes);
        casper_types::U512::from_little_endian(&bytes).into()
    }
}

#[wasm_bindgen]
impl U512 {
    #[wasm_bindgen(js_name = "overflowingMul")]
    pub fn overflowing_mul(&self, other: &Self) -> OverflowingResultU512 {
        let (res, overflow) = self.0.overflowing_mul(other.0);
        OverflowingResultU512 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "overflowingAdd")]
    pub fn overflowing_add(&self, other: &Self) -> OverflowingResultU512 {
        let (res, overflow) = self.0.overflowing_add(other.0);
        OverflowingResultU512 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "overflowingSub")]
    pub fn overflowing_sub(&self, other: &Self) -> OverflowingResultU512 {
        let (res, overflow) = self.0.overflowing_sub(other.0);
        OverflowingResultU512 {
            result: Self(res),
            overflow
        }
    }

    #[wasm_bindgen(js_name = "overflowingPow")]
    pub fn overflowing_pow(&self, exp: u32) -> OverflowingResultU512 {
        let (res, overflow) = self.0.overflowing_pow(*Self::from_u32(exp));
        OverflowingResultU512 {
            result: Self(res),
            overflow
        }
    }
}
