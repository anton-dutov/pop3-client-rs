/// A builder to create a [`Client`] with a connection.
///
/// As it is possible to create the [`Client`] without using `Builder`, we recommend to only use in when you with to define a custom [`ClientConfig`] for the TLS connection.
///
/// [`Client`]: struct.Client
/// [`ClientConfig`]: https://docs.rs/rustls/0.15.2/rustls/struct.ClientConfig.html
pub struct Builder {
    #[cfg(feature = "with-rustls")]
    config: Arc<ClientConfig>,
}

impl Default for Builder {
    #[cfg(not(feature = "with-rustls"))]
    fn default() -> Self {
        Self {}
    }

    #[cfg(feature = "with-rustls")]
    fn default() -> Self {
        let mut config = ClientConfig::new();
        config
            .root_store
            .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);

        let config = Arc::new(config);

        Self { config }
    }
}

impl Builder {
    /// Define a custom config for the TLS connection
    ///
    /// # Example
    /// ```no_run
    /// # use std::result::Result;
    /// # use pop3_client::Builder;
    ///   use rustls::ClientConfig;
    /// #
    /// # fn main() -> Result<(), String> {
    ///
    /// let mut config = ClientConfig::new();
    /// config
    ///     .root_store
    ///     .add_server_trust_anchors(&webpki_roots::TLS_SERVER_ROOTS);
    ///
    /// let client = Builder::default().rustls_config(config).connect("my.host.com", 995)?;
    /// #    Ok(())
    /// # }
    /// ```
    #[cfg(feature = "with-rustls")]
    pub fn rustls_config(&mut self, config: ClientConfig) -> &mut Self {
        self.config = Arc::new(config);
        self
    }
}