use std::time::Duration;

use httpmock::MockServer;

use super::{bot, AuthConfiguration, Configuration, GithubConfiguration, XConfiguration};

mod mocks;

#[tokio::test]
async fn test_x_bot() {
    env_logger::init();
    let mock_server = MockServer::start();

    let mocks = [
        mocks::first_commits_page(&mock_server),
        mocks::second_commits_page(&mock_server),
        mocks::events_to_skip(&mock_server),
        mocks::event_of_different_type(&mock_server),
        mocks::no_new_events(&mock_server),
        mocks::push_event_with_announced_contributor(&mock_server),
        mocks::push_event_with_new_contributor(&mock_server),
        mocks::push_event_with_contributor_just_announced(&mock_server),
        mocks::release_event(&mock_server),
        mocks::verify_release_announced(&mock_server),
        mocks::verify_contributor_announced(&mock_server),
    ];

    let uri = format!("http://{:?}", mock_server.address());

    let test_config = Configuration {
        github: GithubConfiguration {
            base_url: Some(uri.clone()),
            exit_on_poll_error: true,
            poll_frequency: Duration::from_secs(0),
            fetch_pages_per_request: 2,
            ..Default::default()
        },
        x: XConfiguration {
            base_url: Some(uri.clone()),
            ..Default::default()
        },
        auth: AuthConfiguration {
            x_client_identifier: "client_identifier".to_string(),
            x_client_secret: "client_secret".to_string(),
            x_token: "token".to_string(),
            x_token_secret: "token_secret".to_string(),
            github_bearer_token: Some("github_token".to_string()),
        },
    };

    bot::start(test_config).await.unwrap();

    mocks.into_iter().for_each(|mock| mock.assert());
}
