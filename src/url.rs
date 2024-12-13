use std::fmt;

#[derive(Debug)]
pub struct Url {
    pub host: String,
    pub port: Option<u16>,
    pub route: String,
}

impl Url {
    // Constructor for creating a new Url
    pub fn new(host: &str, port: Option<u16>, route: &str) -> Self {
        // Remove leading/trailing whitespace from host and route
        let clean_host = host.trim();
        let clean_route = route.trim();

        // Ensure route starts with '/'
        let formatted_route = if !clean_route.starts_with('/') {
            format!("/{}", clean_route)
        } else {
            clean_route.to_string()
        };

        Url {
            host: clean_host.to_string(),
            port,
            route: formatted_route,
        }
    }

    // Method to format URL with or without HTTPS
    pub fn format(&self, use_https: bool) -> String {
        let protocol = if use_https { "https://" } else { "http://" };

        match self.port {
            Some(port) => format!(
                "{}{}{}{}",
                protocol,
                self.host,
                format!(":{}", port),
                self.route
            ),
            None => format!("{}{}{}", protocol, self.host, self.route),
        }
    }
}

// Implement Display trait for pretty printing
impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.format(true))
    }
}
