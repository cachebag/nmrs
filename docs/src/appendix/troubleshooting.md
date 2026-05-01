# Troubleshooting

Common issues and their solutions when using nmrs.

## Connection Issues

### "D-Bus error" on startup

**Symptom:** `ConnectionError::Dbus` when calling `NetworkManager::new()`

**Causes:**
- NetworkManager is not running
- D-Bus system bus is not accessible
- Insufficient permissions

**Solutions:**

```bash
# Check if NetworkManager is running
systemctl status NetworkManager

# Start NetworkManager if not running
sudo systemctl start NetworkManager
sudo systemctl enable NetworkManager

# Check D-Bus
busctl list | grep NetworkManager
```

### "network not found" (NotFound)

**Symptom:** `ConnectionError::NotFound` when connecting

**Solutions:**
- Verify the SSID is spelled correctly (case-sensitive)
- Trigger a scan first: `nm.scan_networks(None).await?`
- Check if Wi-Fi is enabled: `nm.wifi_state().await?` returns a `RadioState` with `.enabled` and `.hardware_enabled` fields
- Check if the network is in range
- For hidden networks, the network won't appear in scans but should still connect

### "authentication failed" (AuthFailed)

**Symptom:** `ConnectionError::AuthFailed` when connecting

**Solutions:**
- Verify the password is correct
- For WPA-Enterprise, check username format (some networks require `user@domain`, others just `user`)
- Delete the saved profile and retry: `nm.forget("SSID").await?`
- Check if the AP has MAC filtering enabled

### "connection timeout" (Timeout)

**Symptom:** `ConnectionError::Timeout`

**Solutions:**
- Increase the timeout:
  ```rust
  let config = TimeoutConfig::new()
      .with_connection_timeout(Duration::from_secs(60));
  let nm = NetworkManager::with_config(config).await?;
  ```
- Enterprise Wi-Fi (WPA-EAP) often needs longer timeouts
- Check if another connection operation is in progress: `nm.is_connecting().await?`
- Check signal strength — weak signals cause timeouts

### "DHCP failed" (DhcpFailed)

**Symptom:** `ConnectionError::DhcpFailed`

**Solutions:**
- Check if the DHCP server is working (try connecting with another device)
- Try releasing and renewing: disconnect and reconnect
- Check for IP address conflicts on the network

### "no Wi-Fi device found" (NoWifiDevice)

**Symptom:** `ConnectionError::NoWifiDevice`

**Solutions:**

```bash
# Check if a Wi-Fi adapter is detected
ip link show
nmcli device status

# Check if the driver is loaded
lspci -k | grep -A 3 -i network

# Check rfkill
rfkill list
sudo rfkill unblock wifi
```

## VPN Issues

### "invalid WireGuard private key"

**Solutions:**
- Ensure the key is base64-encoded (44 characters, ending in `=`)
- Don't include quotes around the key
- Generate a valid key: `wg genkey`

### "invalid address"

**Solutions:**
- Include CIDR notation: `10.0.0.2/24` (not just `10.0.0.2`)
- Verify the IP is valid

### "invalid VPN gateway"

**Solutions:**
- Use `host:port` format: `vpn.example.com:51820`
- Verify the port is a valid number (1–65535)

### VPN connects but no traffic

**Solutions:**
- Check `allowed_ips` — use `0.0.0.0/0` for full tunnel
- Verify DNS settings: `nm.get_vpn_info("MyVPN").await?.dns_servers`
- Check the WireGuard interface: `ip addr show wg-*`

## Bluetooth Issues

### No Bluetooth devices found

**Solutions:**

```bash
# Check Bluetooth service
systemctl status bluetooth

# Check if adapter is detected
bluetoothctl show

# Make sure the device is paired
bluetoothctl paired-devices
```

- Devices must be **paired** before nmrs can see them
- Use `bluetoothctl pair <MAC>` to pair

## Permission Issues

### PolicyKit errors

If operations fail with permission errors:

```bash
# Check your groups
groups

# Add yourself to the network group
sudo usermod -aG network $USER
# Log out and back in
```

Or create a PolicyKit rule at `/etc/polkit-1/rules.d/50-nmrs.rules`:

```javascript
polkit.addRule(function(action, subject) {
    if (action.id.indexOf("org.freedesktop.NetworkManager.") == 0 &&
        subject.isInGroup("network")) {
        return polkit.Result.YES;
    }
});
```

## Debug Logging

Enable debug logging to diagnose issues:

```bash
RUST_LOG=nmrs=debug cargo run
```

For D-Bus level details:

```bash
RUST_LOG=nmrs=trace,zbus=debug cargo run
```

Monitor NetworkManager's own logs:

```bash
journalctl -u NetworkManager -f
```

## Getting Help

- **Discord:** [discord.gg/Sk3VfrHrN4](https://discord.gg/Sk3VfrHrN4)
- **GitHub Issues:** [github.com/cachebag/nmrs/issues](https://github.com/cachebag/nmrs/issues)
- **API Docs:** [docs.rs/nmrs](https://docs.rs/nmrs)
