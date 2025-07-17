# Troubleshooting Guide – Common Issues and Solutions

> **Audience:** Users experiencing problems with μNet  
> **Status:** Covers current implementation (v0.1.0)

---

## Common CLI Issues

### Command Not Found

**Problem:** `unet: command not found`

**Solutions:**

```bash
# 1. Check if binary exists and is executable
ls -la unet
chmod +x unet

# 2. Use full path
./unet nodes list

# 3. Add to PATH
export PATH=$PATH:$(pwd)
# Or move to system path
sudo mv unet /usr/local/bin/
```

### Database Connection Errors

**Problem:** `Failed to connect to database` or `Database locked`

**Solutions:**

```bash
# 1. Check database file permissions
ls -la unet.db
chmod 644 unet.db

# 2. Ensure directory is writable
ls -la .
# Directory should be writable by current user

# 3. Try different database location
unet --database-url sqlite:///tmp/test.db nodes list

# 4. Check for zombie processes
ps aux | grep unet
# Kill any stuck processes: kill <pid>
```

### Validation Errors

**Problem:** `Invalid input` or `Validation failed`

**Solutions:**

```bash
# 1. Check required fields
unet nodes add --help

# 2. Verify enum values
# Vendor: Cisco, Juniper, Arista, HPE, Dell, etc.
# Role: Core, Distribution, Access, Edge, etc.
# Lifecycle: Planning, Staging, Production, etc.

# 3. Check IP address format
unet nodes add --name test --vendor cisco --model test \
  --role access --lifecycle staging \
  --management-ip 192.168.1.1  # Valid IP format

# 4. Validate UUID format for location-id
unet locations list  # Get valid UUID
```

---

## Server Issues

### Server Won't Start

**Problem:** Server fails to start or crashes immediately

**Solutions:**

```bash
# 1. Check port availability
netstat -an | grep 8080
# Or try different port
unet-server --port 8081

# 2. Check database access
ls -la unet.db
# Ensure server has read/write access

# 3. Verbose logging
unet-server --log-level debug

# 4. Check disk space
df -h .
# Ensure adequate space for database
```

### Server Connection Refused

**Problem:** `Connection refused` when accessing API

**Solutions:**

```bash
# 1. Verify server is running
ps aux | grep unet-server

# 2. Check server logs
unet-server --log-level info

# 3. Test local connection
curl http://localhost:8080/health

# 4. Check firewall rules
# For systemd/firewalld:
sudo firewall-cmd --add-port=8080/tcp

# 5. Bind to all interfaces if needed
unet-server --host 0.0.0.0 --port 8080
```

### API Errors

**Problem:** HTTP 500 errors or unexpected API responses

**Solutions:**

```bash
# 1. Check server logs
unet-server --log-level debug

# 2. Validate request format
curl -X POST http://localhost:8080/api/v1/nodes \
  -H "Content-Type: application/json" \
  -d '{"name":"test","vendor":"Cisco","model":"test","role":"Access","lifecycle":"Staging"}'

# 3. Check database state
sqlite3 unet.db ".tables"
sqlite3 unet.db "SELECT COUNT(*) FROM nodes;"

# 4. Restart server
pkill unet-server
unet-server
```

---

## Policy Issues

### Policy Syntax Errors

**Problem:** `Policy validation failed` or `Parse error`

**Solutions:**

```bash
# 1. Check basic syntax
cat > test.rules << 'EOF'
WHEN node.vendor == "Cisco" THEN SET custom_data.test TO true
EOF

unet policy validate test.rules

# 2. Common syntax fixes:
# - Use quotes around strings: "Cisco" not Cisco
# - Include THEN keyword: WHEN ... THEN ...
# - Check field names: node.vendor not node.brand

# 3. Use verbose validation
unet policy validate policies/ --verbose

# 4. Test with simple policy first
echo 'WHEN true THEN SET custom_data.test TO true' > simple.rules
unet policy validate simple.rules
```

### Policy Evaluation Failures

**Problem:** Policies don't trigger or produce unexpected results

**Solutions:**

```bash
# 1. Check if nodes exist
unet nodes list

# 2. Verify field values
unet nodes show node-name --output json | jq '.data.vendor'

# 3. Test with verbose output
unet policy eval policies/ --verbose

# 4. Test specific node
unet policy eval policies/ --node specific-node-name

# 5. Check custom_data structure
unet nodes show node-name --output json | jq '.data.custom_data'
```

### Policy Results Not Stored

**Problem:** Policy evaluation runs but results aren't saved

**Solutions:**

```bash
# 1. Check API endpoint
curl -X POST http://localhost:8080/api/v1/policies/evaluate \
  -H "Content-Type: application/json" \
  -d '{"store_results": true, "policies": [...]}'

# 2. Verify database tables
sqlite3 unet.db ".schema policy_results"

# 3. Check server permissions
ls -la unet.db
# Server needs write access
```

---

## Data Issues

### Import Failures

**Problem:** `Import failed` or partial imports

**Solutions:**

```bash
# 1. Validate JSON format
cat nodes.json | jq .
# Should show pretty-printed JSON without errors

# 2. Check required fields
cat > minimal-node.json << 'EOF'
{
  "name": "test-node",
  "vendor": "Cisco", 
  "model": "test",
  "role": "Access",
  "lifecycle": "Staging"
}
EOF

unet import minimal-node.json

# 3. Use dry-run to check
unet import data/ --dry-run

# 4. Continue on errors
unet import data/ --continue-on-error

# 5. Import in dependency order
unet import locations.json
unet import nodes.json  
unet import links.json
```

### Export Issues

**Problem:** Export produces empty files or fails

**Solutions:**

```bash
# 1. Check data exists
unet nodes list
unet locations list

# 2. Ensure output directory exists
mkdir -p exports/
unet export --output-dir exports/

# 3. Check permissions
ls -la exports/
# Directory should be writable

# 4. Try different format
unet export --output-dir exports/ --format yaml

# 5. Export specific types
unet export --output-dir exports/ --only nodes
```

---

## Performance Issues

### Slow CLI Commands

**Problem:** Commands take a long time to execute

**Solutions:**

```bash
# 1. Check database size
ls -lh unet.db

# 2. Use pagination for large datasets  
unet nodes list --page 1 --per-page 20

# 3. Add indexes (for large datasets)
sqlite3 unet.db "CREATE INDEX idx_nodes_vendor ON nodes(vendor);"

# 4. Use specific filters
unet nodes list --vendor cisco --role core

# 5. Consider using server mode
unet-server &
export UNET_SERVER=http://localhost:8080
unet nodes list  # Now uses HTTP API
```

### Memory Usage Issues

**Problem:** High memory usage or out-of-memory errors

**Solutions:**

```bash
# 1. Monitor memory usage
ps aux | grep unet | awk '{print $6}' # RSS memory in KB

# 2. Use pagination
unet nodes list --per-page 50

# 3. Limit data returned
unet nodes list --output json | jq '.data.data | length'

# 4. Process in batches
for page in {1..10}; do
  unet nodes list --page $page --per-page 100 --output json
done
```

---

## Network/SNMP Issues

### SNMP Polling Failures

**Problem:** Node status shows as unreachable or no SNMP data

**Solutions:**

```bash
# 1. Check node has management IP
unet nodes show node-name | grep management_ip

# 2. Test SNMP connectivity manually
snmpget -v2c -c public 192.168.1.1 1.3.6.1.2.1.1.1.0

# 3. Check server logs for SNMP errors
unet-server --log-level debug | grep -i snmp

# 4. Verify SNMP configuration on device
# Ensure device has SNMP enabled with proper community/credentials

# 5. Check network connectivity
ping 192.168.1.1
telnet 192.168.1.1 161  # SNMP port
```

### Missing Derived State

**Problem:** Node status or interface data is empty

**Solutions:**

```bash
# 1. Check if SNMP polling is enabled
curl http://localhost:8080/api/v1/nodes/node-id/status

# 2. Verify management IP is set
unet nodes update node-name --management-ip 192.168.1.1

# 3. Check SNMP credentials (when implemented)
# Currently uses default community "public"

# 4. Monitor polling in server logs
unet-server --log-level debug | grep -i "snmp\|polling"

# 5. Check device SNMP configuration
# Ensure device responds to SNMP queries
```

---

## Development Issues

### Build Failures

**Problem:** `cargo build` fails

**Solutions:**

```bash
# 1. Update Rust toolchain
rustup update
rustc --version  # Should be 1.85+

# 2. Clean build cache
cargo clean
cargo build --release

# 3. Check dependencies
cargo check --workspace

# 4. Update dependencies
cargo update

# 5. Check specific feature flags
cargo build --no-default-features
```

### Test Failures

**Problem:** `cargo test` fails

**Solutions:**

```bash
# 1. Run specific test
cargo test test_name

# 2. Run with output
cargo test -- --nocapture

# 3. Test specific package
cargo test -p unet-core

# 4. Clean test databases
rm -f test_*.db

# 5. Check for port conflicts
# Tests may use random ports, conflicts are rare
```

---

## Configuration Issues

### Environment Variables Not Working

**Problem:** Environment variables ignored

**Solutions:**

```bash
# 1. Check variable names
echo $UNET_DATABASE_URL
echo $UNET_SERVER

# 2. Export variables properly
export UNET_DATABASE_URL="sqlite:///path/to/db"

# 3. Use command line flags instead
unet --database-url sqlite:///path/to/db nodes list

# 4. Check precedence order
# CLI flags > Environment variables > Config file > Defaults
```

### Config File Issues

**Problem:** Configuration file not loaded

**Solutions:**

```bash
# 1. Check file location
ls -la ~/.config/unet/config.toml
ls -la ./config.toml

# 2. Validate TOML syntax
cat config.toml | toml verify  # If you have toml CLI tool

# 3. Use explicit config path
unet --config /path/to/config.toml nodes list

# 4. Check file permissions
chmod 644 config.toml
```

---

## Log Analysis

### Enable Debug Logging

```bash
# CLI
unet --verbose nodes list

# Server
unet-server --log-level debug

# Via environment
export UNET_LOG_LEVEL=debug
```

### Common Log Patterns

```bash
# Database issues
grep -i "database\|sqlite\|sea_orm" unet.log

# SNMP issues  
grep -i "snmp\|poll" unet.log

# Policy issues
grep -i "policy\|eval" unet.log

# HTTP issues
grep -i "http\|api\|request" unet.log
```

---

## Getting More Help

### Diagnostic Information

When reporting issues, include:

```bash
# 1. Version information
unet --version
unet-server --version

# 2. System information
uname -a
rustc --version

# 3. Database information
ls -la unet.db
sqlite3 unet.db ".schema" | head -20

# 4. Sample data
unet nodes list --output json | head -50

# 5. Error logs
unet --verbose nodes list 2>&1 | tail -50
```

### Minimal Reproduction

Create minimal test case:

```bash
# 1. Fresh database
rm -f test.db
unet --database-url sqlite://test.db nodes list

# 2. Minimal data
echo '{"name":"test","vendor":"Cisco","model":"test","role":"Access","lifecycle":"Staging"}' > test-node.json
unet --database-url sqlite://test.db import test-node.json

# 3. Reproduce issue
unet --database-url sqlite://test.db nodes show test
```

### Community Support

- **GitHub Issues**: <https://github.com/bedecarroll/unet/issues>
- **Documentation**: <https://unet.bedecarroll.com>
- **Examples**: Check `docs/static/examples/` directory

### Self-Help Resources

1. **CLI Help**: `unet --help`, `unet nodes --help`
2. **API Documentation**: [API Reference](api_reference.md)
3. **Policy Guide**: [Policy Guide](policy_guide.md)
4. **Architecture**: [Architecture Overview](architecture.md)
