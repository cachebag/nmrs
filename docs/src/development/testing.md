# Testing

nmrs includes unit tests, integration tests, and model tests. Since many operations require a running NetworkManager daemon, tests are divided into offline and online categories.

## Running Tests

### Unit Tests

Unit tests cover validation, model construction, and builder logic. They run without NetworkManager:

```bash
cd nmrs
cargo test
```

### Specific Test Modules

```bash
# Model tests
cargo test --lib api::models::tests

# Builder tests
cargo test --lib api::builders

# Validation tests
cargo test --lib util::validation
```

### Integration Tests

Integration tests require a running NetworkManager instance:

```bash
cargo test --test integration_test
cargo test --test validation_test
```

> **Note:** Integration tests that interact with real hardware may fail in CI or on systems without Wi-Fi adapters.

## Test Categories

### Model Tests (`api/models/tests.rs`)

Comprehensive tests for all data types:
- Device type conversions and display formatting
- Device state conversions and transitional state detection
- Wi-Fi security type construction and methods
- EAP options construction (direct and builder)
- VPN credentials construction (direct and builder)
- WireGuard peer configuration
- Bluetooth identity validation
- Timeout config and connection options
- Error type formatting

### Builder Tests

Each builder module includes its own tests:
- `connection_builder.rs` — base settings, IPv4/IPv6 configuration, custom sections
- `wireguard_builder.rs` — WireGuard settings, validation, multiple peers
- `wifi_builder.rs` — Wi-Fi settings, bands, modes

### Validation Tests

`util/validation.rs` tests input validation:
- SSID validation
- Connection name validation
- Wi-Fi security validation (empty passwords, etc.)
- VPN credential validation
- Bluetooth address validation

## Writing Tests

### Offline Tests

For logic that doesn't require D-Bus:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_type_from_code() {
        assert_eq!(DeviceType::from(1), DeviceType::Ethernet);
        assert_eq!(DeviceType::from(2), DeviceType::Wifi);
    }
}
```

### Async Tests

For code that uses async:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_something_async() {
        // Test async logic
    }
}
```

## Docker Testing

For reproducible testing with a real NetworkManager instance:

```bash
docker build -t nmrs-test .
docker run --privileged nmrs-test cargo test
```

The project includes a Dockerfile configured for testing.

## CI/CD

Tests run automatically via GitHub Actions on every push and pull request. The CI workflow:

1. Checks formatting (`cargo fmt --check`)
2. Runs clippy (`cargo clippy`)
3. Runs unit tests (`cargo test`)
4. Builds documentation (`mdbook build`)

## Next Steps

- [Contributing](./contributing.md) – contribution guidelines
- [Architecture](./architecture.md) – understand the codebase
