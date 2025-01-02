use serde::Deserialize;

/// RabbitMQ adapter configuration
#[derive(Debug, Clone, Deserialize)]
pub struct RabbitMQConfig {
    #[serde(default = "default_amqp_addr")]
    pub amqp_addr: String,
    #[serde(default = "default_prefetch_count")]
    pub prefetch_count: u16,
}

fn default_amqp_addr() -> String {
    std::env::var("AMQP_ADDR")
        .map(|addr| {
            // Ensure URL starts with amqp://
            let addr = if !addr.starts_with("amqp://") {
                format!("amqp://{}", addr)
            } else {
                addr
            };

            // Parse the URL to check for vhost
            let without_protocol = addr.trim_start_matches("amqp://");
            if let Some(host_part) = without_protocol.split('@').nth(1) {
                let host_parts: Vec<&str> = host_part.split('/').collect();
                match host_parts.len() {
                    1 => format!("{}/%2F", addr),
                    2 if host_parts[1].is_empty() => format!("{}%2F", addr),
                    2 => addr,
                    _ => addr.split('/').take(4).collect::<Vec<&str>>().join("/"),
                }
            } else {
                format!("{}/%2F", addr)
            }
        })
        .or_else(|_: std::env::VarError| {
            // Try to construct from individual components
            let host = std::env::var("RABBITMQ_HOST").unwrap_or_else(|_| "localhost".to_string());
            let port = std::env::var("RABBITMQ_PORT").unwrap_or_else(|_| "5672".to_string());
            let user = std::env::var("RABBITMQ_USER").unwrap_or_else(|_| "guest".to_string());
            let pass = std::env::var("RABBITMQ_PASSWORD").unwrap_or_else(|_| "guest".to_string());
            let vhost = std::env::var("RABBITMQ_VHOST")
                .unwrap_or_else(|_| "/".to_string())
                .trim_matches('/')
                .to_string();
            
            let vhost_part = if vhost.is_empty() || vhost == "/" {
                "%2F"
            } else {
                &vhost
            };

            Ok::<String, std::env::VarError>(format!("amqp://{}:{}@{}:{}/{}", user, pass, host, port, vhost_part))
        })
        .or_else(|_: std::env::VarError| std::env::var("RABBITMQ_AMQP_ADDR"))
        .unwrap_or_else(|_| "amqp://guest:guest@localhost:5672/%2F".to_string())
}

fn default_prefetch_count() -> u16 {
    1
}

impl Default for RabbitMQConfig {
    fn default() -> Self {
        Self {
            amqp_addr: default_amqp_addr(),
            prefetch_count: default_prefetch_count(),
        }
    }
}

impl RabbitMQConfig {
    /// Validates the configuration
    pub fn validate(&mut self) -> Result<(), String> {
        let mut addr = if self.amqp_addr.is_empty() {
            default_amqp_addr()
        } else {
            self.amqp_addr.clone()
        };

        if addr.is_empty() {
            return Err("AMQP address is not configured".to_string());
        }

        // Ensure URL starts with amqp://
        if !addr.starts_with("amqp://") {
            addr = format!("amqp://{}", addr);
        }

        // Remove the protocol part for easier parsing
        let addr_without_protocol = addr.trim_start_matches("amqp://");

        // Split into credentials and host parts
        let parts: Vec<&str> = addr_without_protocol.split('@').collect();
        if parts.len() != 2 {
            return Err("Invalid AMQP address format. Must be in format: amqp://user:pass@host:port/vhost".to_string());
        }

        // Validate credentials
        let credentials = parts[0];
        if !credentials.contains(':') {
            return Err("Invalid AMQP address format. Must include username and password separated by ':'".to_string());
        }

        // Validate host part and append default vhost if needed
        let host_part = parts[1];
        let host_parts: Vec<&str> = host_part.split('/').collect();
        
        match host_parts.len() {
            1 => {
                // No vhost specified, append default
                if !host_part.contains(':') {
                    return Err("Invalid AMQP address format. Must include port number".to_string());
                }
                addr = format!("{}/%2F", addr);
            },
            2 => {
                // Has vhost, validate host:port
                if !host_parts[0].contains(':') {
                    return Err("Invalid AMQP address format. Must include port number".to_string());
                }
                // Validate vhost is not empty
                if host_parts[1].is_empty() {
                    addr = format!("{}%2F", addr);
                }
            },
            _ => {
                // Too many segments, truncate to first vhost
                addr = addr.split('/').take(4).collect::<Vec<&str>>().join("/");
            }
        }

        // Update the config with the normalized address
        self.amqp_addr = addr;

        Ok(())
    }
} 