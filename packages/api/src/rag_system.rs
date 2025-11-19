// RAG (Retrieval-Augmented Generation) System with Vector Search
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use futures::StreamExt;

use rig::{
    completion::ToolDefinition,
    tool::Tool,
};
use crate::{ChatMessage, ChatRequest, ChatResponse, Role, ToolCall, ToolResult, Tool as ApiTool};

/// Vector embedding
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorEmbedding {
    pub id: String,
    pub vector: Vec<f32>,
    pub content: String,
    pub metadata: DocumentMetadata,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

/// Document metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentMetadata {
    pub source: String,
    pub title: Option<String>,
    pub url: Option<String>,
    pub tags: Vec<String>,
    pub author: Option<String>,
    pub language: Option<String>,
    pub word_count: Option<usize>,
    pub chunk_index: Option<usize>,
    pub total_chunks: Option<usize>,
}

/// Search query
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: String,
    pub top_k: Option<usize>,
    pub threshold: Option<f32>,
    pub filters: Option<HashMap<String, serde_json::Value>>,
}

/// Search result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub document: VectorEmbedding,
    pub score: f32,
    pub relevance_score: f32,
    pub context: String,
}

/// Document chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DocumentChunk {
    pub content: String,
    pub metadata: DocumentMetadata,
    pub chunk_id: String,
}

/// Vector store trait
#[async_trait]
pub trait VectorStore: Send + Sync + std::fmt::Debug {
    /// Add a vector embedding
    async fn add_embedding(&self, embedding: VectorEmbedding) -> Result<()>;

    /// Add multiple embeddings
    async fn add_embeddings(&self, embeddings: Vec<VectorEmbedding>) -> Result<()>;

    /// Search for similar vectors
    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>>;

    /// Delete embedding by ID
    async fn delete_embedding(&self, id: &str) -> Result<()>;

    /// Update embedding
    async fn update_embedding(&self, embedding: VectorEmbedding) -> Result<()>;

    /// Get embedding by ID
    async fn get_embedding(&self, id: &str) -> Result<Option<VectorEmbedding>>;

    /// List all embeddings
    async fn list_embeddings(&self) -> Result<Vec<VectorEmbedding>>;
}

/// Simple in-memory vector store
#[derive(Debug)]
pub struct InMemoryVectorStore {
    embeddings: Arc<RwLock<HashMap<String, VectorEmbedding>>>,
}

impl InMemoryVectorStore {
    pub fn new() -> Self {
        Self {
            embeddings: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn cosine_similarity(vec1: &[f32], vec2: &[f32]) -> f32 {
        let dot_product: f32 = vec1.iter().zip(vec2.iter()).map(|(a, b)| a * b).sum::<f32>();
        let norm1: f32 = vec1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = vec2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            0.0
        } else {
            dot_product / (norm1 * norm2)
        }
    }
}

#[async_trait]
impl VectorStore for InMemoryVectorStore {
    async fn add_embedding(&self, embedding: VectorEmbedding) -> Result<()> {
        let mut embeddings = self.embeddings.write().await;
        embeddings.insert(embedding.id.clone(), embedding);
        Ok(())
    }

    async fn add_embeddings(&self, embeddings: Vec<VectorEmbedding>) -> Result<()> {
        let mut store = self.embeddings.write().await;
        for embedding in embeddings {
            store.insert(embedding.id.clone(), embedding);
        }
        Ok(())
    }

    async fn search(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        let embeddings = self.embeddings.read().await;
        let mut results = Vec::new();

        // Generate mock query vector (in real implementation, would embed the query)
        let query_vector = vec![0.1; 1536]; // Mock 1536-dim vector

        for embedding in embeddings.values() {
            let similarity = Self::cosine_similarity(&query_vector, &embedding.vector);

            // Apply threshold filter
            if let Some(threshold) = query.threshold {
                if similarity < threshold {
                    continue;
                }
            }

            // Apply filters
            let mut passes_filters = true;
            if let Some(filters) = &query.filters {
                for (key, value) in filters {
                    match key.as_str() {
                        "source" => {
                            if embedding.metadata.source != value.as_str().unwrap_or("") {
                                passes_filters = false;
                                break;
                            }
                        },
                        "tags" => {
                            if let Some(expected_tags) = value.as_array() {
                                let expected_tags: Vec<String> = expected_tags
                                    .iter()
                                    .filter_map(|v| v.as_str())
                                    .map(|s| s.to_string())
                                    .collect();

                                let has_all_tags = expected_tags.iter()
                                    .all(|tag| embedding.metadata.tags.contains(tag));

                                if !has_all_tags {
                                    passes_filters = false;
                                    break;
                                }
                            }
                        },
                        _ => {}
                    }
                }
            }

            if passes_filters {
                results.push(SearchResult {
                    document: embedding.clone(),
                    score: similarity,
                    relevance_score: similarity, // In real implementation, would calculate proper relevance
                    context: embedding.content.clone(),
                });
            }
        }

        // Sort by score and apply top_k limit
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        if let Some(top_k) = query.top_k {
            results.truncate(top_k);
        }

        Ok(results)
    }

    async fn delete_embedding(&self, id: &str) -> Result<()> {
        let mut embeddings = self.embeddings.write().await;
        embeddings.remove(id);
        Ok(())
    }

    async fn update_embedding(&self, embedding: VectorEmbedding) -> Result<()> {
        let mut embeddings = self.embeddings.write().await;
        embeddings.insert(embedding.id.clone(), embedding);
        Ok(())
    }

    async fn get_embedding(&self, id: &str) -> Result<Option<VectorEmbedding>> {
        let embeddings = self.embeddings.read().await;
        Ok(embeddings.get(id).cloned())
    }

    async fn list_embeddings(&self) -> Result<Vec<VectorEmbedding>> {
        let embeddings = self.embeddings.read().await;
        Ok(embeddings.values().cloned().collect())
    }
}

/// Document processor
#[derive(Debug)]
pub struct DocumentProcessor {
    chunk_size: usize,
    chunk_overlap: usize,
}

impl DocumentProcessor {
    pub fn new(chunk_size: usize, chunk_overlap: usize) -> Self {
        Self {
            chunk_size,
            chunk_overlap,
        }
    }

    /// Process document into chunks
    pub async fn process_document(
        &self,
        content: String,
        metadata: DocumentMetadata,
    ) -> Result<Vec<DocumentChunk>> {
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut chunks = Vec::new();

        if words.is_empty() {
            return Ok(chunks);
        }

        let chunk_size_words = self.chunk_size;
        let overlap_words = self.chunk_overlap;

        for i in (0..words.len()).step_by(chunk_size_words - overlap_words) {
            let end = std::cmp::min(i + chunk_size_words, words.len());
            let chunk_words = &words[i..end];
            let chunk_content = chunk_words.join(" ");

            let chunk = DocumentChunk {
                content: chunk_content,
                metadata: DocumentMetadata {
                    chunk_index: Some(i / (chunk_size_words - overlap_words)),
                    total_chunks: Some((words.len() + chunk_size_words - overlap_words - 1) / (chunk_size_words - overlap_words)),
                    ..metadata.clone()
                },
                chunk_id: format!("{}_{}", metadata.source, i),
            };

            chunks.push(chunk);
        }

        Ok(chunks)
    }
}

impl Default for DocumentProcessor {
    fn default() -> Self {
        Self::new(500, 50)
    }
}

/// Embedding service trait
#[async_trait]
pub trait EmbeddingService: Send + Sync + std::fmt::Debug {
    /// Generate embedding for text
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>>;

    /// Generate embeddings for multiple texts
    async fn generate_embeddings(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>>;

    /// Get embedding dimension
    fn embedding_dimension(&self) -> usize;
}

/// Mock embedding service
#[derive(Debug)]
pub struct MockEmbeddingService {
    dimension: usize,
}

impl MockEmbeddingService {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingService for MockEmbeddingService {
    async fn generate_embedding(&self, text: &str) -> Result<Vec<f32>> {
        // Generate deterministic mock embedding based on text hash
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        text.hash(&mut hasher);
        let hash_value = hasher.finish();

        let mut embedding = Vec::with_capacity(self.dimension);
        for i in 0..self.dimension {
            let value = ((hash_value >> (i % 64)) % 1000) as f32 / 1000.0;
            embedding.push(value);
        }

        Ok(embedding)
    }

    async fn generate_embeddings(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>> {
        let mut embeddings = Vec::new();
        for text in texts {
            embeddings.push(self.generate_embedding(text).await?);
        }
        Ok(embeddings)
    }

    fn embedding_dimension(&self) -> usize {
        self.dimension
    }
}

/// RAG system
#[derive(Debug)]
pub struct RAGSystem {
    vector_store: Box<dyn VectorStore>,
    embedding_service: Box<dyn EmbeddingService>,
    document_processor: DocumentProcessor,
    prompt_template: String,
}

impl RAGSystem {
    pub fn new(
        vector_store: Box<dyn VectorStore>,
        embedding_service: Box<dyn EmbeddingService>,
    ) -> Self {
        Self {
            vector_store,
            embedding_service,
            document_processor: DocumentProcessor::default(),
            prompt_template: "Based on the following context, answer the user's question:\n\nContext:\n{context}\n\nQuestion: {question}\n\nAnswer:".to_string(),
        }
    }

    /// Add document to RAG system
    pub async fn add_document(
        &self,
        content: String,
        metadata: DocumentMetadata,
    ) -> Result<()> {
        // Process document into chunks
        let chunks = self.document_processor.process_document(content, metadata).await?;

        // Generate embeddings for each chunk
        let chunk_texts: Vec<&str> = chunks.iter().map(|c| c.content.as_str()).collect();
        let embeddings = self.embedding_service.generate_embeddings(&chunk_texts).await?;

        // Create and store embeddings
        let mut vector_embeddings = Vec::new();
        for (chunk, embedding) in chunks.into_iter().zip(embeddings.into_iter()) {
            let vector_embedding = VectorEmbedding {
                id: chunk.chunk_id.clone(),
                vector: embedding,
                content: chunk.content.clone(),
                metadata: chunk.metadata,
                timestamp: chrono::Utc::now(),
            };
            vector_embeddings.push(vector_embedding);
        }

        self.vector_store.add_embeddings(vector_embeddings).await
    }

    /// Retrieve relevant documents
    pub async fn retrieve(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        let search_query = SearchQuery {
            query: query.to_string(),
            top_k: Some(top_k),
            threshold: Some(0.7),
            filters: None,
        };

        self.vector_store.search(search_query).await
    }

    /// Generate RAG-augmented response
    pub async fn generate_response(
        &self,
        query: &str,
        chat_request: ChatRequest,
    ) -> Result<ChatResponse> {
        // Retrieve relevant documents
        let search_results = self.retrieve(query, 3).await?;

        if search_results.is_empty() {
            // No relevant documents found, use regular chat
            return crate::RigAgentService::new()?.send_message(chat_request).await;
        }

        // Build context from search results
        let context: Vec<String> = search_results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                format!("{}. {} (Source: {}, Score: {:.2})",
                    i + 1,
                    result.context,
                    result.document.metadata.source,
                    result.score)
            })
            .collect();

        let context_str = context.join("\n\n");

        // Create enhanced prompt with context
        let enhanced_prompt = self.prompt_template
            .replace("{context}", &context_str)
            .replace("{question}", query);

        // Modify the chat request
        let mut rag_request = chat_request;
        rag_request.system_prompt = Some(enhanced_prompt);

        // Send request with RAG context
        crate::RigAgentService::new()?.send_message(rag_request).await
    }

    /// Search documents
    pub async fn search_documents(&self, query: SearchQuery) -> Result<Vec<SearchResult>> {
        self.vector_store.search(query).await
    }

    /// Delete document
    pub async fn delete_document(&self, source: &str) -> Result<()> {
        let embeddings = self.vector_store.list_embeddings().await?;
        let mut to_delete = Vec::new();

        for embedding in embeddings {
            if embedding.metadata.source == source {
                to_delete.push(embedding.id);
            }
        }

        for id in to_delete {
            self.vector_store.delete_embedding(&id).await?;
        }

        Ok(())
    }

    /// Get statistics
    pub async fn get_statistics(&self) -> Result<RAGStatistics> {
        let embeddings = self.vector_store.list_embeddings().await?;
        let total_documents = embeddings
            .iter()
            .map(|e| &e.metadata.source)
            .collect::<std::collections::HashSet<_>>()
            .len();

        let total_chunks = embeddings.len();
        let total_tokens = embeddings
            .iter()
            .map(|e| e.content.split_whitespace().count())
            .sum();

        Ok(RAGStatistics {
            total_documents,
            total_chunks,
            total_tokens,
            embedding_dimension: self.embedding_service.embedding_dimension(),
        })
    }
}

/// RAG statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RAGStatistics {
    pub total_documents: usize,
    pub total_chunks: usize,
    pub total_tokens: usize,
    pub embedding_dimension: usize,
}

/// RAG tool for rig agents
#[derive(Debug)]
pub struct RAGApiTool {
    rag_system: Arc<RAGSystem>,
}

#[async_trait]
impl Tool for RAGApiTool {
    const NAME: &'static str = "rag_search";
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Args = RAGToolArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> Result<ToolDefinition, Self::Error> {
        Ok(ToolDefinition {
            name: "rag_search".to_string(),
            description: "Search knowledge base using RAG".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "query": {
                        "type": "string",
                        "description": "Search query"
                    },
                    "top_k": {
                        "type": "integer",
                        "description": "Number of results to return",
                        "default": 3
                    }
                },
                "required": ["query"]
            }),
        })
    }

    async fn call(&self, args: serde_json::Value) -> Result<Self::Output, Self::Error> {
        let args: RAGToolArgs = serde_json::from_value(args)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        let search_query = SearchQuery {
            query: args.query,
            top_k: args.top_k,
            threshold: Some(0.7),
            filters: None,
        };

        let results = self.rag_system.search_documents(search_query).await?;

        if results.is_empty() {
            return Ok("No relevant information found in the knowledge base.".to_string());
        }

        let response = results
            .iter()
            .enumerate()
            .map(|(i, result)| {
                format!("{}. {} (Score: {:.2})\nSource: {}",
                    i + 1,
                    result.context,
                    result.score,
                    result.document.metadata.source)
            })
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(response)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct RAGToolArgs {
    pub query: String,
    pub top_k: Option<usize>,
}

/// Knowledge base management tool
#[derive(Debug)]
pub struct KnowledgeBaseApiTool {
    rag_system: Arc<RAGSystem>,
}

#[async_trait]
impl Tool for KnowledgeBaseApiTool {
    const NAME: &'static str = "knowledge_base";
    type Error = Box<dyn std::error::Error + Send + Sync>;
    type Args = KnowledgeBaseArgs;
    type Output = String;

    async fn definition(&self, _prompt: String) -> Result<ToolDefinition, Self::Error> {
        Ok(ToolDefinition {
            name: "knowledge_base".to_string(),
            description: "Manage knowledge base documents".to_string(),
            parameters: json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["add", "delete", "stats", "search"],
                        "description": "Action to perform"
                    },
                    "content": {
                        "type": "string",
                        "description": "Document content (for add action)"
                    },
                    "source": {
                        "type": "string",
                        "description": "Source identifier (for add/delete action)"
                    },
                    "query": {
                        "type": "string",
                        "description": "Search query (for search action)"
                    }
                },
                "required": ["action"]
            }),
        })
    }

    async fn call(&self, args: serde_json::Value) -> Result<Self::Output, Self::Error> {
        let args: KnowledgeBaseArgs = serde_json::from_value(args)
            .map_err(|e| -> Box<dyn std::error::Error + Send + Sync> { Box::new(e) })?;

        match args.action.as_str() {
            "add" => {
                if let (Some(content), Some(source)) = (&args.content, &args.source) {
                    let metadata = DocumentMetadata {
                        source: source.clone(),
                        title: None,
                        url: None,
                        tags: vec!["user_added".to_string()],
                        author: None,
                        language: None,
                        word_count: Some(content.split_whitespace().count()),
                        chunk_index: None,
                        total_chunks: None,
                    };

                    self.rag_system.add_document(content.clone(), metadata).await?;
                    Ok(format!("Document '{}' added to knowledge base.", source))
                } else {
                    Ok("Error: content and source are required for add action.".to_string())
                }
            },
            "delete" => {
                if let Some(source) = &args.source {
                    self.rag_system.delete_document(source).await?;
                    Ok(format!("Document '{}' deleted from knowledge base.", source))
                } else {
                    Ok("Error: source is required for delete action.".to_string())
                }
            },
            "stats" => {
                let stats = self.rag_system.get_statistics().await?;
                Ok(format!("Knowledge Base Statistics:\n- Documents: {}\n- Chunks: {}\n- Tokens: {}\n- Embedding Dimension: {}",
                    stats.total_documents, stats.total_chunks, stats.total_tokens, stats.embedding_dimension))
            },
            "search" => {
                if let Some(query) = &args.query {
                    let search_query = SearchQuery {
                        query: query.clone(),
                        top_k: args.top_k,
                        threshold: Some(0.7),
                        filters: None,
                    };

                    let results = self.rag_system.search_documents(search_query).await?;
                    if results.is_empty() {
                        Ok("No documents found matching the query.".to_string())
                    } else {
                        let response = results
                            .iter()
                            .enumerate()
                            .map(|(i, result)| {
                                format!("{}. {} (Score: {:.2})", i + 1, result.context, result.score)
                            })
                            .collect::<Vec<_>>()
                            .join("\n\n");
                        Ok(response)
                    }
                } else {
                    Ok("Error: query is required for search action.".to_string())
                }
            },
            _ => Ok("Error: Unknown action.".to_string())
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct KnowledgeBaseArgs {
    pub action: String,
    pub content: Option<String>,
    pub source: Option<String>,
    pub query: Option<String>,
    pub top_k: Option<usize>,
}

/// Enhanced agent service with RAG support
pub struct RAGEnabledAgentService {
    base_service: crate::RigAgentService,
    rag_system: Arc<RAGSystem>,
    rag_tools: Vec<ApiTool>,
}

impl RAGEnabledAgentService {
    pub fn new() -> Result<Self> {
        let vector_store = Box::new(InMemoryVectorStore::new());
        let embedding_service = Box::new(MockEmbeddingService::new(1536));
        let rag_system = Arc::new(RAGSystem::new(vector_store, embedding_service));

        let rag_tools = vec![
            ApiTool {
                name: "rag_search".to_string(),
                description: "Search knowledge base using RAG".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "query": {"type": "string"},
                        "top_k": {"type": "integer"}
                    },
                    "required": ["query"]
                }),
                is_mcp: false,
            },
            ApiTool {
                name: "knowledge_base".to_string(),
                description: "Manage knowledge base documents".to_string(),
                input_schema: json!({
                    "type": "object",
                    "properties": {
                        "action": {"type": "string"},
                        "content": {"type": "string"},
                        "source": {"type": "string"}
                    },
                    "required": ["action"]
                }),
                is_mcp: false,
            },
        ];

        Ok(Self {
            base_service: crate::RigAgentService::new()?,
            rag_system,
            rag_tools,
        })
    }

    /// Send message with RAG enhancement
    pub async fn send_message_with_rag(&self, request: ChatRequest) -> Result<ChatResponse> {
        // Add RAG tools to the request
        let mut enhanced_request = request;
        let mut tools = enhanced_request.tools.unwrap_or_default();
        tools.extend(self.rag_tools.clone());
        enhanced_request.tools = Some(tools);

        // For now, just send to base service with enhanced tools
        // In a full implementation, would check if query requires RAG
        self.base_service.send_message(enhanced_request).await
    }

    /// Add document to knowledge base
    pub async fn add_document(&self, content: String, metadata: DocumentMetadata) -> Result<()> {
        self.rag_system.add_document(content, metadata).await
    }

    /// Search knowledge base
    pub async fn search_knowledge_base(&self, query: &str, top_k: usize) -> Result<Vec<SearchResult>> {
        self.rag_system.retrieve(query, top_k).await
    }

    /// Get RAG statistics
    pub async fn get_rag_statistics(&self) -> Result<RAGStatistics> {
        self.rag_system.get_statistics().await
    }
}

impl Default for RAGEnabledAgentService {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

// Implement delegation for base service methods
impl std::ops::Deref for RAGEnabledAgentService {
    type Target = crate::RigAgentService;

    fn deref(&self) -> &Self::Target {
        &self.base_service
    }
}