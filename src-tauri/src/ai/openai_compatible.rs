use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EmbeddingRequest {
    pub model: String,
    pub input: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingResult {
    pub vectors: Vec<Vec<f32>>,
    pub dimension: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProviderErrorCode {
    ConfigMissing,
    AuthFailed,
    HttpError,
    NetworkError,
    ModelResponseInvalid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderError {
    pub code: ProviderErrorCode,
    pub message: String,
    pub status: Option<u16>,
}

impl ProviderError {
    pub fn config_missing(message: impl Into<String>) -> Self {
        Self {
            code: ProviderErrorCode::ConfigMissing,
            message: message.into(),
            status: None,
        }
    }

    fn network(message: impl Into<String>) -> Self {
        Self {
            code: ProviderErrorCode::NetworkError,
            message: message.into(),
            status: None,
        }
    }

    fn invalid_response(message: impl Into<String>) -> Self {
        Self {
            code: ProviderErrorCode::ModelResponseInvalid,
            message: message.into(),
            status: None,
        }
    }

    fn from_http(status: u16, body: &str) -> Self {
        let message = parse_error_message(body)
            .filter(|message| !message.trim().is_empty())
            .unwrap_or_else(|| format!("provider returned HTTP {status}"));
        Self {
            code: if matches!(status, 401 | 403) {
                ProviderErrorCode::AuthFailed
            } else {
                ProviderErrorCode::HttpError
            },
            message,
            status: Some(status),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiRequest {
    pub url: String,
    pub bearer_token: String,
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenAiResponse {
    pub status: u16,
    pub body: String,
}

pub trait OpenAiTransport {
    fn post_json(&self, request: OpenAiRequest) -> Result<OpenAiResponse, ProviderError>;
}

pub struct OpenAiCompatibleClient<T> {
    base_url: String,
    api_key: String,
    transport: T,
}

impl<T> OpenAiCompatibleClient<T>
where
    T: OpenAiTransport,
{
    pub fn new(base_url: String, api_key: String, transport: T) -> Self {
        Self {
            base_url,
            api_key,
            transport,
        }
    }

    pub fn transport(&self) -> &T {
        &self.transport
    }

    pub fn chat(
        &self,
        model: &str,
        messages: Vec<ChatMessage>,
        temperature: f32,
    ) -> Result<String, ProviderError> {
        let request = ChatRequest {
            model: model.to_string(),
            messages,
            temperature,
        };
        let body = serde_json::to_string(&request)
            .map_err(|error| ProviderError::invalid_response(error.to_string()))?;
        let response = self.transport.post_json(OpenAiRequest {
            url: chat_url(&self.base_url),
            bearer_token: self.api_key.clone(),
            body,
        })?;
        ensure_success(response).and_then(|body| parse_chat_content(&body).map_err(invalid_json))
    }

    pub fn embeddings(
        &self,
        model: &str,
        input: Vec<String>,
    ) -> Result<EmbeddingResult, ProviderError> {
        let request = EmbeddingRequest {
            model: model.to_string(),
            input,
        };
        let body = serde_json::to_string(&request)
            .map_err(|error| ProviderError::invalid_response(error.to_string()))?;
        let response = self.transport.post_json(OpenAiRequest {
            url: embeddings_url(&self.base_url),
            bearer_token: self.api_key.clone(),
            body,
        })?;
        let vectors = ensure_success(response).and_then(|body| parse_embeddings(&body).map_err(invalid_json))?;
        let dimension = vectors.first().map_or(0, Vec::len);
        Ok(EmbeddingResult { vectors, dimension })
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub struct UreqOpenAiTransport;

impl OpenAiTransport for UreqOpenAiTransport {
    fn post_json(&self, request: OpenAiRequest) -> Result<OpenAiResponse, ProviderError> {
        let body_json: serde_json::Value = serde_json::from_str(&request.body)
            .map_err(|error| ProviderError::invalid_response(error.to_string()))?;
        let agent: ureq::Agent = ureq::Agent::config_builder()
            .http_status_as_error(false)
            .build()
            .into();
        let response = agent
            .post(&request.url)
            .header("Authorization", &format!("Bearer {}", request.bearer_token))
            .header("Content-Type", "application/json")
            .send_json(body_json)
            .map_err(|error| ProviderError::network(error.to_string()))?;
        let status = response.status().as_u16();
        let body = response
            .into_body()
            .read_to_string()
            .map_err(|error| ProviderError::network(error.to_string()))?;
        Ok(OpenAiResponse { status, body })
    }
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Debug, Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingDatum>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingDatum {
    embedding: Vec<f32>,
}

#[derive(Debug, Deserialize)]
struct ErrorEnvelope {
    error: Option<ErrorBody>,
}

#[derive(Debug, Deserialize)]
struct ErrorBody {
    message: Option<String>,
}

pub fn chat_url(base_url: &str) -> String {
    format!("{}/chat/completions", base_url.trim_end_matches('/'))
}

pub fn embeddings_url(base_url: &str) -> String {
    format!("{}/embeddings", base_url.trim_end_matches('/'))
}

pub fn parse_chat_content(json: &str) -> Result<String, serde_json::Error> {
    let response: ChatResponse = serde_json::from_str(json)?;
    Ok(response
        .choices
        .first()
        .map(|choice| choice.message.content.clone())
        .unwrap_or_default())
}

pub fn parse_embeddings(json: &str) -> Result<Vec<Vec<f32>>, serde_json::Error> {
    let response: EmbeddingResponse = serde_json::from_str(json)?;
    Ok(response.data.into_iter().map(|datum| datum.embedding).collect())
}

fn ensure_success(response: OpenAiResponse) -> Result<String, ProviderError> {
    if (200..300).contains(&response.status) {
        Ok(response.body)
    } else {
        Err(ProviderError::from_http(response.status, &response.body))
    }
}

fn invalid_json(error: serde_json::Error) -> ProviderError {
    ProviderError::invalid_response(error.to_string())
}

fn parse_error_message(body: &str) -> Option<String> {
    serde_json::from_str::<ErrorEnvelope>(body)
        .ok()
        .and_then(|envelope| envelope.error)
        .and_then(|error| error.message)
}
