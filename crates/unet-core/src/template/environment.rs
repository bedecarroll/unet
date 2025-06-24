//! Template environment configuration and management

use anyhow::{Context, Result};
use minijinja::{Environment, Value};
use tracing::{debug, info};

/// Template environment wrapper with μNet-specific configuration
#[derive(Debug)]
pub struct TemplateEnvironment {
    env: Environment<'static>,
}

impl TemplateEnvironment {
    /// Create a new template environment with μNet defaults
    pub fn new() -> Result<Self> {
        let mut env = Environment::new();

        // Configure environment settings - disable auto-escaping for network configs
        env.set_auto_escape_callback(|_name| minijinja::AutoEscape::None);

        // Set strict undefined behavior
        env.set_undefined_behavior(minijinja::UndefinedBehavior::Strict);

        // Security: Disable potentially dangerous features
        env.set_debug(false);

        // Add custom filters and functions
        Self::register_network_filters(&mut env)?;
        Self::register_network_functions(&mut env)?;

        info!(
            "Template environment initialized with network-specific filters and security restrictions"
        );

        Ok(Self { env })
    }

    /// Register network-specific Jinja filters
    fn register_network_filters(env: &mut Environment<'static>) -> Result<()> {
        // IP address manipulation filters
        env.add_filter(
            "ip_network",
            |ip: String, prefix: u8| -> Result<String, minijinja::Error> {
                Ok(format!("{}/{}", ip, prefix))
            },
        );

        env.add_filter(
            "ip_netmask",
            |prefix: u8| -> Result<String, minijinja::Error> {
                let mask = match prefix {
                    8 => "255.0.0.0",
                    9 => "255.128.0.0",
                    10 => "255.192.0.0",
                    11 => "255.224.0.0",
                    12 => "255.240.0.0",
                    13 => "255.248.0.0",
                    14 => "255.252.0.0",
                    15 => "255.254.0.0",
                    16 => "255.255.0.0",
                    17 => "255.255.128.0",
                    18 => "255.255.192.0",
                    19 => "255.255.224.0",
                    20 => "255.255.240.0",
                    21 => "255.255.248.0",
                    22 => "255.255.252.0",
                    23 => "255.255.254.0",
                    24 => "255.255.255.0",
                    25 => "255.255.255.128",
                    26 => "255.255.255.192",
                    27 => "255.255.255.224",
                    28 => "255.255.255.240",
                    29 => "255.255.255.248",
                    30 => "255.255.255.252",
                    31 => "255.255.255.254",
                    32 => "255.255.255.255",
                    _ => {
                        return Err(minijinja::Error::new(
                            minijinja::ErrorKind::InvalidOperation,
                            format!("Unsupported prefix length: {}", prefix),
                        ));
                    }
                };
                Ok(mask.to_string())
            },
        );

        env.add_filter(
            "ip_wildcard",
            |prefix: u8| -> Result<String, minijinja::Error> {
                let wildcard = match prefix {
                    8 => "0.255.255.255",
                    9 => "0.127.255.255",
                    10 => "0.63.255.255",
                    11 => "0.31.255.255",
                    12 => "0.15.255.255",
                    13 => "0.7.255.255",
                    14 => "0.3.255.255",
                    15 => "0.1.255.255",
                    16 => "0.0.255.255",
                    17 => "0.0.127.255",
                    18 => "0.0.63.255",
                    19 => "0.0.31.255",
                    20 => "0.0.15.255",
                    21 => "0.0.7.255",
                    22 => "0.0.3.255",
                    23 => "0.0.1.255",
                    24 => "0.0.0.255",
                    25 => "0.0.0.127",
                    26 => "0.0.0.63",
                    27 => "0.0.0.31",
                    28 => "0.0.0.15",
                    29 => "0.0.0.7",
                    30 => "0.0.0.3",
                    31 => "0.0.0.1",
                    32 => "0.0.0.0",
                    _ => {
                        return Err(minijinja::Error::new(
                            minijinja::ErrorKind::InvalidOperation,
                            format!("Unsupported prefix length: {}", prefix),
                        ));
                    }
                };
                Ok(wildcard.to_string())
            },
        );

        // String formatting helpers for configs
        env.add_filter("indent", |text: String, spaces: usize| -> String {
            let indent = " ".repeat(spaces);
            text.lines()
                .map(|line| {
                    if line.is_empty() {
                        line.to_string()
                    } else {
                        format!("{}{}", indent, line)
                    }
                })
                .collect::<Vec<_>>()
                .join("\n")
        });

        env.add_filter("cisco_interface", |interface: String| -> String {
            // Convert interface names to Cisco format
            interface
                .replace("GigabitEthernet", "Gi")
                .replace("TenGigabitEthernet", "Te")
                .replace("FastEthernet", "Fa")
                .replace("Ethernet", "Et")
        });

        env.add_filter("juniper_interface", |interface: String| -> String {
            // Convert interface names to Juniper format
            interface
                .replace("GigabitEthernet", "ge-")
                .replace("TenGigabitEthernet", "xe-")
                .replace("FastEthernet", "fe-")
        });

        env.add_filter("uppercase", |text: String| -> String {
            text.to_uppercase()
        });

        env.add_filter("lowercase", |text: String| -> String {
            text.to_lowercase()
        });

        env.add_filter(
            "mac_format",
            |mac: String, separator: Option<String>| -> Result<String, minijinja::Error> {
                let sep = separator.unwrap_or_else(|| ":".to_string());
                let clean_mac = mac.replace([':', '-', '.'], "");
                if clean_mac.len() != 12 {
                    return Err(minijinja::Error::new(
                        minijinja::ErrorKind::InvalidOperation,
                        format!("Invalid MAC address format: {}", mac),
                    ));
                }
                let formatted = clean_mac
                    .chars()
                    .collect::<Vec<_>>()
                    .chunks(2)
                    .map(|chunk| chunk.iter().collect::<String>())
                    .collect::<Vec<_>>()
                    .join(&sep);
                Ok(formatted)
            },
        );

        debug!("Registered network-specific filters");
        Ok(())
    }

    /// Register network-specific Jinja functions
    fn register_network_functions(env: &mut Environment<'static>) -> Result<()> {
        // Network calculation utilities
        env.add_function("vlan_range", |start: u16, end: u16| -> Vec<u16> {
            (start..=end).collect()
        });

        env.add_function(
            "interface_range",
            |prefix: String, start: u8, end: u8| -> Vec<String> {
                (start..=end).map(|i| format!("{}{}", prefix, i)).collect()
            },
        );

        env.add_function("port_range", |start: u16, end: u16| -> Vec<u16> {
            (start..=end).collect()
        });

        env.add_function("asn_private", || -> Vec<u32> {
            // Return common private ASN ranges
            let mut asns = Vec::new();
            asns.extend(64512..=65534); // RFC 6996 private ASNs
            asns
        });

        env.add_function(
            "subnet_hosts",
            |prefix: u8| -> Result<u32, minijinja::Error> {
                if prefix > 32 {
                    return Err(minijinja::Error::new(
                        minijinja::ErrorKind::InvalidOperation,
                        format!("Invalid prefix length: {}", prefix),
                    ));
                }
                let host_bits = 32 - prefix;
                let hosts = if host_bits >= 2 {
                    (1u32 << host_bits) - 2 // Subtract network and broadcast
                } else {
                    0
                };
                Ok(hosts)
            },
        );

        env.add_function(
            "ip_increment",
            |ip: String, increment: u32| -> Result<String, minijinja::Error> {
                // Simple IP increment (IPv4 only)
                let parts: Vec<&str> = ip.split('.').collect();
                if parts.len() != 4 {
                    return Err(minijinja::Error::new(
                        minijinja::ErrorKind::InvalidOperation,
                        format!("Invalid IP address format: {}", ip),
                    ));
                }

                let mut ip_num = 0u32;
                for (i, part) in parts.iter().enumerate() {
                    let octet: u32 = part.parse().map_err(|_| {
                        minijinja::Error::new(
                            minijinja::ErrorKind::InvalidOperation,
                            format!("Invalid IP octet: {}", part),
                        )
                    })?;
                    ip_num |= octet << (8 * (3 - i));
                }

                ip_num = ip_num.wrapping_add(increment);

                let result = format!(
                    "{}.{}.{}.{}",
                    (ip_num >> 24) & 0xFF,
                    (ip_num >> 16) & 0xFF,
                    (ip_num >> 8) & 0xFF,
                    ip_num & 0xFF
                );
                Ok(result)
            },
        );

        debug!("Registered network-specific functions");
        Ok(())
    }

    /// Get the underlying MiniJinja environment
    pub fn inner(&self) -> &Environment<'static> {
        &self.env
    }

    /// Get a mutable reference to the underlying environment
    pub fn inner_mut(&mut self) -> &mut Environment<'static> {
        &mut self.env
    }

    /// Render a template string with the given context
    pub fn render_str(&self, template_str: &str, context: &Value) -> Result<String> {
        let template = self
            .env
            .template_from_str(template_str)
            .context("Failed to parse template")?;

        template
            .render(context)
            .context("Failed to render template")
    }

    /// Check if a template is valid without rendering it
    pub fn validate_template(&self, template_str: &str) -> Result<()> {
        self.env
            .template_from_str(template_str)
            .context("Template validation failed")?;
        Ok(())
    }
}

impl Default for TemplateEnvironment {
    fn default() -> Self {
        Self::new().expect("Failed to create default TemplateEnvironment")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_template_environment_creation() {
        let env = TemplateEnvironment::new();
        assert!(env.is_ok());
    }

    #[test]
    fn test_network_filters() {
        let env = TemplateEnvironment::new().unwrap();
        let context = Value::from_serialize(json!({
            "ip": "192.168.1.1",
            "prefix": 24,
            "interface": "GigabitEthernet0/0/1",
            "mac": "aa:bb:cc:dd:ee:ff",
            "text": "hello world"
        }));

        // IP network filter
        let template = "{{ ip | ip_network(prefix) }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "192.168.1.1/24");

        // IP netmask filter
        let template = "{{ prefix | ip_netmask }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "255.255.255.0");

        // IP wildcard filter
        let template = "{{ prefix | ip_wildcard }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "0.0.0.255");

        // Cisco interface filter
        let template = "{{ interface | cisco_interface }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "Gi0/0/1");

        // Juniper interface filter
        let template = "{{ interface | juniper_interface }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "ge-0/0/1");

        // Case conversion filters
        let template = "{{ text | uppercase }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "HELLO WORLD");

        let template = "{{ text | lowercase }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "hello world");

        // MAC format filter
        let template = "{{ mac | mac_format }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "aa:bb:cc:dd:ee:ff");

        let template = "{{ mac | mac_format('-') }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "aa-bb-cc-dd-ee-ff");
    }

    #[test]
    fn test_network_functions() {
        let env = TemplateEnvironment::new().unwrap();
        let context = Value::from_serialize(json!({}));

        // VLAN range function
        let template = "{% for vlan in vlan_range(10, 12) %}{{ vlan }}{% if not loop.last %},{% endif %}{% endfor %}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "10,11,12");

        // Interface range function
        let template = "{% for iface in interface_range('Gi0/0/', 1, 3) %}{{ iface }}{% if not loop.last %},{% endif %}{% endfor %}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "Gi0/0/1,Gi0/0/2,Gi0/0/3");

        // Port range function
        let template = "{% for port in port_range(8080, 8082) %}{{ port }}{% if not loop.last %},{% endif %}{% endfor %}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "8080,8081,8082");

        // Subnet hosts function
        let template = "{{ subnet_hosts(24) }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "254");

        let template = "{{ subnet_hosts(30) }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "2");

        // IP increment function
        let template = "{{ ip_increment('192.168.1.1', 10) }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "192.168.1.11");

        let template = "{{ ip_increment('192.168.1.250', 10) }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "192.168.2.4");
    }

    #[test]
    fn test_indent_filter() {
        let env = TemplateEnvironment::new().unwrap();
        let context = Value::from_serialize(json!({
            "text": "line1\nline2\nline3"
        }));

        let template = "{{ text | indent(4) }}";
        let result = env.render_str(template, &context).unwrap();
        assert_eq!(result, "    line1\n    line2\n    line3");
    }

    #[test]
    fn test_template_validation() {
        let env = TemplateEnvironment::new().unwrap();

        // Valid template
        assert!(env.validate_template("Hello {{ name }}!").is_ok());

        // Invalid template (unclosed tag)
        assert!(env.validate_template("Hello {{ name }!").is_err());
    }
}
