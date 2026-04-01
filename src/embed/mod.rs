use ort::session::{builder::GraphOptimizationLevel, Session};
use ort::value::Tensor;
use std::sync::Mutex;
use tokenizers::Tokenizer;

use crate::error::{KbError, Result};

mod model;

pub use model::download_model;

pub const EMBEDDING_DIM: usize = 384;

pub struct Embedder {
    session: Mutex<Session>,
    tokenizer: Mutex<Tokenizer>,
}

unsafe impl Send for Embedder {}
unsafe impl Sync for Embedder {}

impl Embedder {
    pub fn new(model_dir: &std::path::Path) -> Result<Self> {
        let model_file = model_dir.join("model.onnx");
        let tokenizer_file = model_dir.join("tokenizer.json");

        if !model_file.exists() {
            return Err(KbError::EmbeddingError(format!(
                "model file not found: {}",
                model_file.display()
            )));
        }
        if !tokenizer_file.exists() {
            return Err(KbError::EmbeddingError(format!(
                "tokenizer file not found: {}",
                tokenizer_file.display()
            )));
        }

        let session = Session::builder()
            .map_err(|e| KbError::EmbeddingErrorOrt(e.to_string()))?
            .with_optimization_level(GraphOptimizationLevel::Level3)
            .map_err(|e| KbError::EmbeddingErrorOrt(e.to_string()))?
            .with_intra_threads(4)
            .map_err(|e| KbError::EmbeddingErrorOrt(e.to_string()))?
            .commit_from_file(&model_file)
            .map_err(|e| KbError::EmbeddingErrorOrt(e.to_string()))?;

        let tokenizer = Tokenizer::from_file(&tokenizer_file)
            .map_err(|e| KbError::EmbeddingError(format!("failed to load tokenizer: {}", e)))?;

        Ok(Self {
            session: Mutex::new(session),
            tokenizer: Mutex::new(tokenizer),
        })
    }

    pub fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let encoding = self
            .tokenizer
            .lock()
            .unwrap()
            .encode(text, true)
            .map_err(|e| KbError::EmbeddingError(format!("tokenization failed: {}", e)))?;

        let input_ids: Vec<i64> = encoding.get_ids().iter().map(|x| *x as i64).collect();
        let attention_mask: Vec<i64> = encoding
            .get_attention_mask()
            .iter()
            .map(|x| *x as i64)
            .collect();
        let token_type_ids: Vec<i64> = vec![0; input_ids.len()];

        let input_ids_shape = vec![1, input_ids.len()];
        let attention_mask_shape = vec![1, attention_mask.len()];
        let token_type_ids_shape = vec![1, input_ids.len()];

        let input_ids_tensor = Tensor::from_array((input_ids_shape, input_ids))
            .map_err(|e| KbError::EmbeddingErrorOrt(e.to_string()))?;
        let attention_mask_tensor = Tensor::from_array((attention_mask_shape, attention_mask))
            .map_err(|e| KbError::EmbeddingErrorOrt(e.to_string()))?;
        let token_type_ids_tensor = Tensor::from_array((token_type_ids_shape, token_type_ids))
            .map_err(|e| KbError::EmbeddingErrorOrt(e.to_string()))?;

        let last_hidden_state = {
            let mut session = self.session.lock().unwrap();
            let outputs = session
                .run(ort::inputs![
                    "input_ids" => input_ids_tensor,
                    "attention_mask" => attention_mask_tensor,
                    "token_type_ids" => token_type_ids_tensor,
                ])
                .map_err(|e| KbError::EmbeddingErrorOrt(e.to_string()))?;

            let last_hidden_state = &outputs["last_hidden_state"];
            let (dims, embedding_data) = last_hidden_state
                .try_extract_tensor::<f32>()
                .map_err(|e| KbError::EmbeddingError(format!("failed to extract output: {}", e)))?;
            if dims.len() != 3 {
                return Err(KbError::EmbeddingError(format!(
                    "expected 3D output, got {}D",
                    dims.len()
                )));
            }
            if dims[2] as usize != EMBEDDING_DIM {
                return Err(KbError::EmbeddingError(format!(
                    "expected embedding dim {}, got {}",
                    EMBEDDING_DIM, dims[2]
                )));
            }

            embedding_data
                .iter()
                .take(EMBEDDING_DIM)
                .copied()
                .collect::<Vec<f32>>()
        };

        let norm: f32 = last_hidden_state.iter().map(|x| x * x).sum::<f32>().sqrt();
        let embedding: Vec<f32> = if norm > 0.0 {
            last_hidden_state.iter().map(|x| x / norm).collect()
        } else {
            last_hidden_state
        };

        Ok(embedding)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_embed_produces_correct_dimensions() {
        let model_dir = std::path::Path::new("/tmp/kb_models");
        if !model_dir.join("model.onnx").exists() {
            return;
        }
        let mut embedder = Embedder::new(model_dir).unwrap();
        let embedding = embedder.embed("hello world").unwrap();
        assert_eq!(embedding.len(), EMBEDDING_DIM);
    }
}
