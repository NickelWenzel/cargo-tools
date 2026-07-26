pub(crate) fn has_features(feature_count: usize) -> bool {
    feature_count > 0
}

pub(crate) fn has_multiple_features(feature_count: usize) -> bool {
    feature_count > 1
}

pub(crate) fn show_features_node(features_enabled: bool, feature_count: usize) -> bool {
    features_enabled && has_features(feature_count)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_visibility_depends_on_feature_count() {
        assert!(!has_features(0));
        assert!(has_features(1));
        assert!(has_features(2));

        assert!(!has_multiple_features(0));
        assert!(!has_multiple_features(1));
        assert!(has_multiple_features(2));

        assert!(!show_features_node(false, 1));
        assert!(!show_features_node(true, 0));
        assert!(show_features_node(true, 1));
    }
}
