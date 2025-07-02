use std::sync::Arc;
use anyhow::Result;
use tracing::{info, debug, warn};
use serde::{Serialize, Deserialize};

use crate::db::{Database, SearchResult};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridSearchResult {
    pub id: i64,
    pub content: String,
    pub timestamp: u64,
    pub text_score: f64,
    pub semantic_score: f64,
    pub combined_score: f64,
    pub context: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub limit: usize,
    pub text_weight: f64,
    pub semantic_weight: f64,
    pub min_score_threshold: f64,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            limit: 50,
            text_weight: 0.7,
            semantic_weight: 0.3,
            min_score_threshold: 0.1,
        }
    }
}

pub struct SearchEngine {
    database: Arc<Database>,
    embedding_model: Option<EmbeddingModel>,
}

// Placeholder for embedding model - will be implemented with rust-bert
struct EmbeddingModel {
    // TODO: Implement with rust-bert
}

impl EmbeddingModel {
    fn new() -> Result<Self> {
        // TODO: Initialize rust-bert model
        Ok(Self {})
    }

    fn encode(&self, _text: &str) -> Result<Vec<f32>> {
        // TODO: Implement actual embedding generation
        // For now, return a dummy embedding
        warn!("🚧 Usando embedding dummy - implementar rust-bert");
        Ok(vec![0.1; 384]) // Dummy 384-dimensional embedding
    }

    fn similarity(&self, embedding1: &[f32], embedding2: &[f32]) -> f64 {
        // Cosine similarity
        if embedding1.len() != embedding2.len() {
            return 0.0;
        }

        let dot_product: f32 = embedding1.iter()
            .zip(embedding2.iter())
            .map(|(a, b)| a * b)
            .sum();

        let norm1: f32 = embedding1.iter().map(|x| x * x).sum::<f32>().sqrt();
        let norm2: f32 = embedding2.iter().map(|x| x * x).sum::<f32>().sqrt();

        if norm1 == 0.0 || norm2 == 0.0 {
            return 0.0;
        }

        (dot_product / (norm1 * norm2)) as f64
    }
}

impl SearchEngine {
    pub async fn new(database: Arc<Database>) -> Result<Self> {
        info!("🔍 Inicializando motor de busca...");

        // Try to initialize embedding model
        let embedding_model = match EmbeddingModel::new() {
            Ok(model) => {
                info!("✅ Modelo de embeddings inicializado");
                Some(model)
            },
            Err(e) => {
                warn!("⚠️ Falha ao inicializar modelo de embeddings: {}. Busca semântica será limitada.", e);
                None
            }
        };

        Ok(Self {
            database,
            embedding_model,
        })
    }

    pub async fn search_text(&self, query: &str, options: &SearchOptions) -> Result<Vec<SearchResult>> {
        debug!("🔍 Executando busca textual para: {}", query);

        self.database.search_text(query, options.limit).await
    }

    pub async fn search_semantic(&self, query: &str, options: &SearchOptions) -> Result<Vec<HybridSearchResult>> {
        debug!("🧠 Executando busca semântica para: {}", query);

        let embedding_model = match &self.embedding_model {
            Some(model) => model,
            None => {
                warn!("⚠️ Modelo de embeddings não disponível para busca semântica");
                return Ok(Vec::new());
            }
        };

        // Generate embedding for query
        let query_embedding = embedding_model.encode(query)?;

        // Get all events with embeddings (this is a simplified approach)
        // In production, you'd want to use a proper vector database or indexing
        let all_events = self.database.search_by_timerange(0, u64::MAX, 10000).await?;

        let mut semantic_results = Vec::new();

        for event in all_events {
            if let Some(content) = &event.text_content {
                if content.trim().is_empty() {
                    continue;
                }

                // Try to get existing embedding
                let event_embedding = match self.database.get_embedding(event.id).await? {
                    Some(embedding) => embedding,
                    None => {
                        // Generate and store embedding
                        let embedding = embedding_model.encode(content)?;
                        if let Err(e) = self.database.store_embedding(event.id, &embedding).await {
                            warn!("⚠️ Falha ao armazenar embedding para evento {}: {}", event.id, e);
                        }
                        embedding
                    }
                };

                // Calculate similarity
                let similarity = embedding_model.similarity(&query_embedding, &event_embedding);

                if similarity >= options.min_score_threshold {
                    semantic_results.push(HybridSearchResult {
                        id: event.id,
                        content: content.clone(),
                        timestamp: event.timestamp,
                        text_score: 0.0,
                        semantic_score: similarity,
                        combined_score: similarity,
                        context: event.application,
                    });
                }
            }
        }

        // Sort by semantic score
        semantic_results.sort_by(|a, b| b.semantic_score.partial_cmp(&a.semantic_score).unwrap());
        semantic_results.truncate(options.limit);

        debug!("🧠 Busca semântica retornou {} resultados", semantic_results.len());
        Ok(semantic_results)
    }

    pub async fn search_hybrid(&self, query: &str, options: &SearchOptions) -> Result<Vec<HybridSearchResult>> {
        debug!("🔍🧠 Executando busca híbrida para: {}", query);

        // Perform both text and semantic search
        let text_results = self.search_text(query, options).await?;
        let semantic_results = self.search_semantic(query, options).await?;

        // Combine results using Reciprocal Rank Fusion (RRF)
        let combined_results = self.combine_results_rrf(text_results, semantic_results, options);

        debug!("🔍🧠 Busca híbrida retornou {} resultados", combined_results.len());
        Ok(combined_results)
    }

    fn combine_results_rrf(&self, text_results: Vec<SearchResult>, semantic_results: Vec<HybridSearchResult>, options: &SearchOptions) -> Vec<HybridSearchResult> {
        use std::collections::HashMap;

        let mut combined_scores: HashMap<i64, (f64, f64, String, u64, Option<String>)> = HashMap::new();

        // Add text search scores
        for (rank, result) in text_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f64 + 1.0); // RRF with k=60
            combined_scores.insert(result.id, (
                rrf_score * options.text_weight,
                0.0,
                result.content.clone(),
                result.timestamp,
                result.context.clone()
            ));
        }

        // Add semantic search scores
        for (rank, result) in semantic_results.iter().enumerate() {
            let rrf_score = 1.0 / (60.0 + rank as f64 + 1.0); // RRF with k=60
            let semantic_score = rrf_score * options.semantic_weight;

            combined_scores.entry(result.id)
                .and_modify(|(_text_score, sem_score, _, _, _)| *sem_score = semantic_score)
                .or_insert((0.0, semantic_score, result.content.clone(), result.timestamp, result.context.clone()));
        }

        // Convert to final results and sort
        let mut final_results: Vec<HybridSearchResult> = combined_scores
            .into_iter()
            .map(|(id, (text_score, semantic_score, content, timestamp, context))| {
                HybridSearchResult {
                    id,
                    content,
                    timestamp,
                    text_score,
                    semantic_score,
                    combined_score: text_score + semantic_score,
                    context,
                }
            })
            .filter(|result| result.combined_score >= options.min_score_threshold)
            .collect();

        final_results.sort_by(|a, b| b.combined_score.partial_cmp(&a.combined_score).unwrap());
        final_results.truncate(options.limit);

        final_results
    }

    pub async fn get_search_suggestions(&self, partial_query: &str, limit: usize) -> Result<Vec<String>> {
        // Implementação básica usando busca textual
        match self.database.search_text(partial_query, limit).await {
            Ok(results) => {
                let suggestions: Vec<String> = results
                    .into_iter()
                    .map(|r| r.content)
                    .take(limit)
                    .collect();
                Ok(suggestions)
            },
            Err(e) => {
                warn!("Erro ao gerar sugestões: {}", e);
                Ok(vec![])
            }
        }
    }

    pub async fn get_popular_searches(&self, limit: usize) -> Result<Vec<String>> {
        debug!("📊 Obtendo buscas populares");

        // This would typically be based on search analytics
        // For now, return some common patterns
        Ok(vec![
            "email".to_string(),
            "password".to_string(),
            "login".to_string(),
            "document".to_string(),
            "meeting".to_string(),
        ].into_iter().take(limit).collect())
    }

    pub async fn optimize_search_index(&self) -> Result<()> {
        info!("🔧 Otimizando índices de busca...");

        // Optimize database
        self.database.vacuum().await?;

        // TODO: Optimize vector index if using a proper vector database

        info!("✅ Índices de busca otimizados");
        Ok(())
    }


}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use crate::agent::KeyEvent;

    #[tokio::test]
    async fn test_search_engine_creation() {
        let temp_file = NamedTempFile::new().unwrap();
        let database = Arc::new(Database::new(temp_file.path()).await.unwrap());
        let search_engine = SearchEngine::new(database).await.unwrap();

        // Test basic functionality
        let options = SearchOptions::default();
        let results = search_engine.search_text("test", &options).await.unwrap();
        assert_eq!(results.len(), 0); // Empty database
    }

    #[tokio::test]
    async fn test_search_options_default() {
        let options = SearchOptions::default();
        assert_eq!(options.limit, 50);
        assert_eq!(options.text_weight, 0.7);
        assert_eq!(options.semantic_weight, 0.3);
        assert_eq!(options.min_score_threshold, 0.1);
    }
}
