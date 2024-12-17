use std::sync::atomic::AtomicUsize;
use std::sync::atomic::Ordering::Relaxed;

use httpmock::{prelude::*, Method, Mock, Then, When};

fn etag_header_not_exists(request: &HttpMockRequest) -> bool {
    request
        .headers
        .iter()
        .flat_map(|h| h.iter())
        .filter(|(k, _)| k == "if-none-match")
        .count()
        == 0
}

static ETAG_SEQ: AtomicUsize = AtomicUsize::new(0);

fn next_etag(then: Then) -> Then {
    then.header("etag", format!("W/\"{:?}\"", ETAG_SEQ.load(Relaxed)))
}

fn match_etag(when: When) -> When {
    when.header(
        "if-none-match",
        format!("W/\"{:?}\"", ETAG_SEQ.fetch_add(1, Relaxed)),
    )
}

pub fn first_commits_page(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/repos/delta-rs/delta/commits")
                .query_param("sha", "master")
                .query_param("per_page", "2")
                .query_param("page", "0")
                .method(Method::GET);

            then.status(200)
                .body_from_file("test_resources/mock_first_commits_page.json")
                .header(
                    "link",
                    "<https://next; rel=\"next\", <https://last; rel=\"last\"",
                );
        })
}

pub fn second_commits_page(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/repos/delta-rs/delta/commits")
                .query_param("sha", "master")
                .query_param("per_page", "2")
                .query_param("page", "2")
                .method(Method::GET);
            then.status(200)
                .body_from_file("test_resources/mock_second_commits_page.json");
        })
}

pub fn events_to_skip(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/repos/delta-rs/delta/events")
                .matches(etag_header_not_exists)
                .query_param("per_page", "2")
                .method(Method::GET);
            then.status(200)
                .and(next_etag)
                .body_from_file("test_resources/mock_release_event.json");
        })
}

pub fn push_event_with_announced_contributor(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/repos/delta-rs/delta/events")
                .and(match_etag)
                .query_param("per_page", "2")
                .method(Method::GET);
            then.status(200)
                .and(next_etag)
                .body_from_file("test_resources/mock_push_event_with_announced_contributor.json");
        })
}

pub fn push_event_with_new_contributor(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/repos/delta-rs/delta/events")
                .and(match_etag)
                .query_param("per_page", "2")
                .method(Method::GET);
            then.status(200)
                .and(next_etag)
                .body_from_file("test_resources/mock_push_event_with_new_contributor.json");
        })
}

pub fn push_event_with_contributor_just_announced(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/repos/delta-rs/delta/events")
                .and(match_etag)
                .query_param("per_page", "2")
                .method(Method::GET);
            then.status(200)
                .and(next_etag)
                .body_from_file("test_resources/mock_push_event_with_new_contributor.json");
        })
}

pub fn event_of_different_type(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/repos/delta-rs/delta/events")
                .and(match_etag)
                .query_param("per_page", "2")
                .method(Method::GET);
            then.status(200)
                .and(next_etag)
                .body_from_file("test_resources/mock_delete_event.json");
        })
}

pub fn no_new_events(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/repos/delta-rs/delta/events")
                .and(match_etag)
                .query_param("per_page", "2")
                .method(Method::GET);
            then.status(304).and(next_etag);
        })
}

pub fn release_event(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/repos/delta-rs/delta/events")
                .and(match_etag)
                .query_param("per_page", "2")
                .method(Method::GET);
            then.status(200)
                .and(next_etag)
                .body_from_file("test_resources/mock_release_event.json");
        })
}

pub fn verify_contributor_announced(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/2/tweets")
                .method(Method::POST)
                .body_contains("NEW CONTRIBUTOR");
            then.status(200);
        })
}

pub fn verify_release_announced(mock_server: &MockServer) -> Mock<'_> {
    mock_server
        .mock(|when, then| {
            when.path("/2/tweets")
                .method(Method::POST)
                .body_contains("RELEASE VERSION");
            then.status(200);
        })
}

