//! Provider-neutral search-and-retrieve orchestration.
//!
//! The client attempts a bounded number of search results sequentially, preserving
//! successful exact-byte resources and typed failure summaries. It never executes,
//! automatically saves, or injects retrieved material into live chat.

use crate::web_retrieval::{
    BoundedRetriever, RetrievalError, RetrievedResource, SearchProvider, WebFetchRequest,
    WebSearchHit, WebSearchRequest,
};
use serde::{Deserialize, Serialize};

const HARD_MAX_FETCHES: u8 = 20;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchRetrievalPolicy {
    pub max_fetches: u8,
    pub max_successes: u8,
}

impl Default for SearchRetrievalPolicy {
    fn default() -> Self {
        Self {
            max_fetches: 8,
            max_successes: 5,
        }
    }
}

impl SearchRetrievalPolicy {
    pub fn validate(&self) -> Result<(), RetrievalError> {
        if self.max_fetches == 0 || self.max_fetches > HARD_MAX_FETCHES {
            return Err(RetrievalError::SearchProvider(format!(
                "max_fetches must be between 1 and {HARD_MAX_FETCHES}"
            )));
        }
        if self.max_successes == 0 || self.max_successes > self.max_fetches {
            return Err(RetrievalError::SearchProvider(
                "max_successes must be between 1 and max_fetches".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct SearchFetchOutcome {
    pub hit: WebSearchHit,
    pub resource: Option<RetrievedResource>,
    pub error: Option<String>,
}

impl SearchFetchOutcome {
    pub fn succeeded(&self) -> bool {
        self.resource.is_some()
    }
}

pub struct WebResearchClient<P> {
    provider: P,
    retriever: BoundedRetriever,
    policy: SearchRetrievalPolicy,
}

impl<P> WebResearchClient<P>
where
    P: SearchProvider,
{
    pub fn new(
        provider: P,
        retriever: BoundedRetriever,
        policy: SearchRetrievalPolicy,
    ) -> Result<Self, RetrievalError> {
        policy.validate()?;
        Ok(Self {
            provider,
            retriever,
            policy,
        })
    }

    pub fn provider(&self) -> &P {
        &self.provider
    }

    pub fn retriever(&self) -> &BoundedRetriever {
        &self.retriever
    }

    pub fn policy(&self) -> &SearchRetrievalPolicy {
        &self.policy
    }

    pub fn search_and_retrieve(
        &self,
        request: &WebSearchRequest,
    ) -> Result<Vec<SearchFetchOutcome>, RetrievalError> {
        request.validate()?;
        let hits = self.provider.search(request)?;
        let mut outcomes = Vec::new();
        let mut successes = 0u8;

        for hit in hits.into_iter().take(self.policy.max_fetches as usize) {
            let fetch = self.retriever.fetch(&WebFetchRequest {
                url: hit.url.clone(),
                purpose: format!(
                    "retrieve public search result rank {} for user-authorized research",
                    hit.rank
                ),
            });
            match fetch {
                Ok(resource) => {
                    successes += 1;
                    outcomes.push(SearchFetchOutcome {
                        hit,
                        resource: Some(resource),
                        error: None,
                    });
                    if successes >= self.policy.max_successes {
                        break;
                    }
                }
                Err(error) => outcomes.push(SearchFetchOutcome {
                    hit,
                    resource: None,
                    error: Some(error.to_string()),
                }),
            }
        }
        Ok(outcomes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::web_retrieval::{RetrievalPolicy, WebSearchHit};
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    struct FixedProvider {
        hits: Vec<WebSearchHit>,
    }

    impl SearchProvider for FixedProvider {
        fn search(&self, request: &WebSearchRequest) -> Result<Vec<WebSearchHit>, RetrievalError> {
            request.validate()?;
            Ok(self.hits.clone())
        }
    }

    fn spawn_server(responses: Vec<String>) -> String {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind research server");
        let address = listener.local_addr().expect("research server address");
        thread::spawn(move || {
            for response in responses {
                let (mut stream, _) = listener.accept().expect("accept retrieval request");
                let mut request = [0u8; 2_048];
                let _ = stream.read(&mut request);
                stream
                    .write_all(response.as_bytes())
                    .expect("write retrieval response");
            }
        });
        format!("http://{address}")
    }

    #[test]
    fn preserves_failures_and_continues_to_later_results() {
        let body = "retrieved evidence";
        let base = spawn_server(vec![
            "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                .to_string(),
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            ),
        ]);
        let provider = FixedProvider {
            hits: vec![
                WebSearchHit {
                    rank: 1,
                    url: format!("{base}/missing"),
                    title: "Missing".to_string(),
                    snippet: None,
                },
                WebSearchHit {
                    rank: 2,
                    url: format!("{base}/evidence"),
                    title: "Evidence".to_string(),
                    snippet: None,
                },
            ],
        };
        let retriever = BoundedRetriever::new(RetrievalPolicy {
            max_bytes: 1_024,
            timeout_ms: 2_000,
            max_redirects: 1,
            allow_http: true,
            allow_private_network: true,
            ..RetrievalPolicy::default()
        })
        .expect("valid retriever");
        let client = WebResearchClient::new(
            provider,
            retriever,
            SearchRetrievalPolicy {
                max_fetches: 2,
                max_successes: 2,
            },
        )
        .expect("valid research client");
        let outcomes = client
            .search_and_retrieve(&WebSearchRequest {
                query: "evidence".to_string(),
                max_results: 2,
            })
            .expect("research succeeds");
        assert_eq!(outcomes.len(), 2);
        assert!(!outcomes[0].succeeded());
        assert!(outcomes[0].error.is_some());
        assert!(outcomes[1].succeeded());
        assert_eq!(
            outcomes[1]
                .resource
                .as_ref()
                .expect("resource")
                .as_text()
                .expect("text"),
            body
        );
    }
}
