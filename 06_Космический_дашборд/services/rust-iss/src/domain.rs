use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn same_coordinates_have_zero_distance() {
        assert_eq!(haversine_km(55.0, 37.0, 55.0, 37.0), 0.0);
    }
    #[test]
    fn equator_one_degree() {
        assert!((haversine_km(0.0, 0.0, 0.0, 1.0) - 111.195).abs() < 0.01);
    }
    #[test]
    fn numbers_accept_strings_but_not_missing_values() {
        assert_eq!(num(&serde_json::json!("12.5")), Some(12.5));
        assert_eq!(num(&serde_json::json!(12.5)), Some(12.5));
        assert_eq!(num(&serde_json::Value::Null), None);
    }
}
pub(crate) fn num(v: &Value) -> Option<f64> {
    if let Some(x) = v.as_f64() { return Some(x); }
    if let Some(s) = v.as_str() { return s.parse::<f64>().ok(); }
    None
}

pub(crate) fn haversine_km(lat1: f64, lon1: f64, lat2: f64, lon2: f64) -> f64 {
    let rlat1 = lat1.to_radians();
    let rlat2 = lat2.to_radians();
    let dlat = (lat2 - lat1).to_radians();
    let dlon = (lon2 - lon1).to_radians();
    let a = (dlat / 2.0).sin().powi(2) + rlat1.cos() * rlat2.cos() * (dlon / 2.0).sin().powi(2);
    let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
    6371.0 * c
}

