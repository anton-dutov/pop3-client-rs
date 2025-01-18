#[cfg(test)]
#[cfg(feature = "runtime-tokio")]
mod tests {
    use pop3_client::*;


    async fn tokio_connect() -> Result<AsyncClient> {
        AsyncClient::connect("pop3.mailtrap.io", 1100).await
    }

    #[cfg(feature = "with-rustls")]
    fn connect() -> Result<Client> {
        pop3_client::Builder::default().connect("pop3.mailtrap.io", 1100)
    }

    #[tokio::test]
    async fn tokio_connects() {
        assert!(tokio_connect().await.is_ok());
    }

    #[tokio::test]
    async fn login_success() {
        let mut client = tokio_connect().await.unwrap();
        let result = client.login("e913202b66b623", "1ddf1a9bd7fc45").await;
        eprintln!("login_success: {:?}", result);
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn login_wrong_login() {
        let mut client = tokio_connect().await.unwrap();
        let result = client.login("e913202b66b62", "1ddf1a9bd7fc45").await;
        eprintln!("wrong_login: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    #[tokio::test]
    async fn login_wrong_password() {
        let mut client = tokio_connect().await.unwrap();
        let result = client.login("e913202b66b623", "1ddf1a9bd7fc4").await;
        eprintln!("wrong_password: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    #[tokio::test]
    async fn login_wrong_stage() {
        let mut client = tokio_connect().await.unwrap();
        client.login("e913202b66b623", "1ddf1a9bd7fc45").await.ok();
        let result = client.login("e913202b66b623", "1ddf1a9bd7fc45").await;
        eprintln!("login_wrong_stage: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    // This test will fail if the server implementation does not comply to specification
    #[tokio::test]
    #[ignore]
    async fn login_already_locked() {
        tokio_connect()
            .await
            .unwrap()
            .login("e913202b66b623", "1ddf1a9bd7fc45")
            .await
            .ok();
        let mut client = tokio_connect().await.unwrap();
        let result = client.login("e913202b66b623", "1ddf1a9bd7fc45").await;
        eprintln!("login_already_locked: {:?}", result);
        assert!(result.is_err())
    }

    #[tokio::test]
    async fn quit() {
        tokio_connect().await.unwrap().quit().await.unwrap()
    }

    #[tokio::test]
    async fn stat_success() {
        let mut client = tokio_connect().await.unwrap();
        client.login("e913202b66b623", "1ddf1a9bd7fc45").await.ok();
        let result = client.stat().await;
        eprintln!("stat_success: {:?}", result);
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn stat_wrong_stage() {
        let mut client = tokio_connect().await.unwrap();
        let result = client.stat().await;
        eprintln!("stat_wrong_stage: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    #[tokio::test]
    async fn list_all() {
        let mut client = tokio_connect().await.unwrap();
        client.login("e913202b66b623", "1ddf1a9bd7fc45").await.ok();
        let result = client.list(None).await;
        eprintln!("list_all: {:?}", result);
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn list_wrong_stage()
    {
        let mut client = tokio_connect().await.unwrap();
        let result = client.list(None).await;
        eprintln!("list_wrong_stage: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    #[tokio::test]
    async fn retr_not_found() {
        let mut client = tokio_connect().await.unwrap();
        client.login("e913202b66b623", "1ddf1a9bd7fc45").await.ok();
        let result = client.retr(8).await;
        eprintln!("retr_not_found: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    #[tokio::test]
    async fn retr_wrong_stage() {
        let mut client = tokio_connect().await.unwrap();
        let result = client.retr(10).await;
        eprintln!("retr_wrong_stage: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    #[tokio::test]
    async fn dele_not_found() {
        let mut client = tokio_connect().await.unwrap();
        client.login("e913202b66b623", "1ddf1a9bd7fc45").await.ok();
        let result = client.dele(8).await;
        eprintln!("dele_not_found: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    #[tokio::test]
    async fn dele_wrong_stage()
    {
        let mut client = tokio_connect().await.unwrap();
        let result = client.dele(10).await;
        eprintln!("dele_wrong_stage: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    #[tokio::test]
    async fn noop_success()
    {
        let mut client = tokio_connect().await.unwrap();
        client.login("e913202b66b623", "1ddf1a9bd7fc45").await.ok();
        let result = client.noop().await;
        eprintln!("noop_success: {:?}", result);
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn rset_all() {
        let mut client = tokio_connect().await.unwrap();
        client.login("e913202b66b623", "1ddf1a9bd7fc45").await.ok();
        let result = client.rset().await;
        eprintln!("rset_success: {:?}", result);
        assert!(result.is_ok())
    }

    #[tokio::test]
    async fn rset_wrong_stage() {
        let mut client = tokio_connect().await.unwrap();
        let result = client.rset().await;
        eprintln!("rset_wrong_stage: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }


    #[tokio::test]
    async fn top_not_found() {
        let mut client = tokio_connect().await.unwrap();
        client.login("e913202b66b623", "1ddf1a9bd7fc45").await.ok();
        let result = client.top(8, 3).await;
        eprintln!("top_not_found: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }

    #[tokio::test]
    async fn top_wrong_stage() {
        let mut client = tokio_connect().await.unwrap();
        let result = client.top(10, 4).await;
        eprintln!("top_wrong_stage: {:?}", result);
        assert!(result.is_err());
        assert!(!matches!(result.unwrap_err(), Pop3Error::ConnectionClosed))
    }
}
