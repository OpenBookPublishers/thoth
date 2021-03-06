#[macro_export]
macro_rules! graphql_query_builder {
    (
        $request:ident,
        $request_body:ident,
        $variables: ty,
        $query:expr,
        $response_body:ident,
        $response_data:ty,
        $fetch:ident,
        $fetch_action:ident
    ) => {
        use yewtil::fetch::Fetch;
        use yewtil::fetch::FetchAction;
        use yewtil::fetch::FetchRequest;
        use yewtil::fetch::Json;
        use yewtil::fetch::MethodBody;

        use crate::THOTH_API;

        pub type $fetch = Fetch<$request, $response_body>;
        pub type $fetch_action = FetchAction<$response_body>;

        #[derive(Default, Debug, Clone)]
        pub struct $request {
            pub body: $request_body,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct $request_body {
            pub query: String,
            pub variables: $variables,
        }

        #[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
        pub struct $response_body {
            pub data: $response_data,
        }

        impl FetchRequest for $request {
            type RequestBody = $request_body;
            type ResponseBody = $response_body;
            type Format = Json;

            fn url(&self) -> String {
                format!("{}/graphql", THOTH_API)
            }

            fn method(&self) -> MethodBody<Self::RequestBody> {
                MethodBody::Post(&self.body)
            }

            fn headers(&self) -> Vec<(String, String)> {
                use crate::service::account::AccountService;

                let account_service = AccountService::new();
                let json = ("Content-Type".into(), "application/json".into());
                if let Some(token) = account_service.get_token() {
                    let auth = ("Authorization".into(), format!("Bearer {}", token));
                    vec![json, auth]
                } else {
                    vec![json]
                }
            }

            fn use_cors(&self) -> bool {
                true
            }
        }

        impl Default for $request_body {
            fn default() -> $request_body {
                $request_body {
                    query: $query.to_string(),
                    variables: Default::default(),
                }
            }
        }

        impl Default for $response_body {
            fn default() -> $response_body {
                $response_body {
                    data: Default::default(),
                }
            }
        }
    };
}

pub mod contribution;
pub mod contributor;
pub mod funder;
pub mod funding;
pub mod imprint;
pub mod issue;
pub mod language;
pub mod price;
pub mod publication;
pub mod publisher;
pub mod series;
pub mod stats;
pub mod subject;
pub mod work;
