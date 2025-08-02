use core::{Config, CoreService};

#[tokio::test]
async fn test_server_functionality() {
    let config = Config::default();
    let service = CoreService::new(config);

    let result = service.process_data("test data").unwrap();
    assert!(result.contains("cargo-tools-test"));
    assert!(result.contains("test data"));
}

#[tokio::test]
async fn test_server_config() {
    let config = Config {
        name: "test-server".to_string(),
        version: "1.0.0".to_string(),
    };
    let service = CoreService::new(config);

    assert_eq!(service.get_config().name, "test-server");
    assert_eq!(service.get_config().version, "1.0.0");
}
