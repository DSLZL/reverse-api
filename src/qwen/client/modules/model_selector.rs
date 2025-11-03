use crate::qwen::models::Model;

pub struct ModelSelector;

impl ModelSelector {
    /// Check if a model supports thinking capability
    pub fn model_supports_thinking(models: &[Model], model_id: &str) -> bool {
        models
            .iter()
            .find(|m| m.id == model_id)
            .and_then(|m| m.info.as_ref())
            .map(|info| info.meta.capabilities.thinking)
            .unwrap_or(false)
    }

    /// Check if a model supports search capability
    pub fn model_supports_search(models: &[Model], model_id: &str) -> bool {
        models
            .iter()
            .find(|m| m.id == model_id)
            .and_then(|m| m.info.as_ref())
            .map(|info| info.meta.chat_type.contains(&"search".to_string()))
            .unwrap_or(false)
    }

    /// Get maximum thinking budget for a model
    pub fn get_model_thinking_budget(models: &[Model], model_id: &str) -> Option<u32> {
        models
            .iter()
            .find(|m| m.id == model_id)
            .and_then(|m| m.info.as_ref())
            .and_then(|info| {
                if info.meta.capabilities.thinking_budget {
                    Some(info.meta.max_thinking_generation_length)
                } else {
                    None
                }
            })
    }

    /// Get all models that support thinking
    pub fn get_thinking_capable_models(models: Vec<Model>) -> Vec<Model> {
        models
            .into_iter()
            .filter(|m| {
                m.info
                    .as_ref()
                    .map(|info| info.meta.capabilities.thinking)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get all models that support search
    pub fn get_search_capable_models(models: Vec<Model>) -> Vec<Model> {
        models
            .into_iter()
            .filter(|m| {
                m.info
                    .as_ref()
                    .map(|info| info.meta.chat_type.contains(&"search".to_string()))
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get all models that support vision
    pub fn get_vision_capable_models(models: Vec<Model>) -> Vec<Model> {
        models
            .into_iter()
            .filter(|m| {
                m.info
                    .as_ref()
                    .map(|info| info.meta.capabilities.vision)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get all models that support audio
    pub fn get_audio_capable_models(models: Vec<Model>) -> Vec<Model> {
        models
            .into_iter()
            .filter(|m| {
                m.info
                    .as_ref()
                    .map(|info| info.meta.capabilities.audio)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Get all models that support video
    pub fn get_video_capable_models(models: Vec<Model>) -> Vec<Model> {
        models
            .into_iter()
            .filter(|m| {
                m.info
                    .as_ref()
                    .map(|info| info.meta.capabilities.video)
                    .unwrap_or(false)
            })
            .collect()
    }

    /// Smart model selection based on requirements
    pub fn select_best_model(
        models: Vec<Model>,
        requires_vision: bool,
        requires_audio: bool,
        requires_video: bool,
        requires_thinking: bool,
        requires_search: bool,
    ) -> String {
        let mut scored_models: Vec<(String, u32)> = models
            .into_iter()
            .filter_map(|model| {
                let info = model.info.as_ref()?;
                let caps = &info.meta.capabilities;
                let mut score = 0u32;

                if requires_vision && !caps.vision {
                    return None;
                }
                if requires_audio && !caps.audio {
                    return None;
                }
                if requires_video && !caps.video {
                    return None;
                }
                if requires_thinking && !caps.thinking {
                    return None;
                }
                if requires_search && !info.meta.chat_type.contains(&"search".to_string()) {
                    return None;
                }

                if requires_vision && caps.vision {
                    score += 100;
                }
                if requires_audio && caps.audio {
                    score += 100;
                }
                if requires_video && caps.video {
                    score += 100;
                }
                if requires_thinking && caps.thinking {
                    score += 100;
                }
                if requires_search && info.meta.chat_type.contains(&"search".to_string()) {
                    score += 100;
                }

                if caps.document {
                    score += 10;
                }
                if caps.citations {
                    score += 10;
                }

                score += (info.meta.max_context_length / 10000).min(50);

                Some((model.id, score))
            })
            .collect();

        scored_models.sort_by(|a, b| b.1.cmp(&a.1));

        scored_models
            .first()
            .map(|(id, _)| id.clone())
            .unwrap_or_else(|| "qwen3-max".to_string())
    }
}
