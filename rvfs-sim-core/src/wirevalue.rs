//! A clamped float representing the values a simulated Wire can hold.

/// Representation of the values which a Wire can take between low (0.0) and high (1.0).
#[derive(Debug, Copy, Clone, PartialEq, PartialOrd)]
pub struct WireValue {
    /// Wire level value, in the range [0.0, 1.0].
    level: f32,
}

impl WireValue {
    /// Create a new WireValue with the value clamped to the permitted range.
    pub fn new(level: f32) -> Self {
        Self {
            level: level.clamp(0.0, 1.0),
        }
    }
}

impl From<f32> for WireValue {
    /// Convert a float to a WireValue.  The float value will be clamped to the acceptable range.
    fn from(item: f32) -> WireValue {
        WireValue::new(item)
    }
}

impl From<WireValue> for f32 {
    /// Convert a WireValue to a float in the range [0.0, 1.0].
    fn from(item: WireValue) -> f32 {
        item.level
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wire_value_create() {
        // GIVEN a float in the valid wire range
        let value = 0.2f32;
        // WHEN a new wire value is created from that float
        let wv = WireValue::new(value);
        // THEN the wire value level equals that float value
        assert_eq!(value, wv.level);
    }
    #[test]
    fn wire_value_create_too_large() {
        // GIVEN a float larger than the valid wire range
        let value = 7.3f32;
        // WHEN a new wire value is created from that float
        let wv = WireValue::new(value);
        // THEN the wire value level is limited to the maximum of 1.0
        assert_eq!(1.0, wv.level);
    }
    #[test]
    fn wire_value_create_too_small() {
        // GIVEN a float smaller than the valid wire range
        let value = -7.3f32;
        // WHEN a new wire value is created from that float
        let wv = WireValue::new(value);
        // THEN the wire value level is limited to the minimum of 0.0
        assert_eq!(0.0, wv.level);
    }
    #[test]
    fn wire_value_from_float() {
        // GIVEN a float in the valid wire range
        let value = 0.2f32;
        // WHEN that float is converted to a wire value
        let wv = WireValue::from(value);
        // THEN the wire value level equals that float value
        assert_eq!(value, wv.level);
    }
}
