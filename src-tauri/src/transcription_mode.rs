//! Stable transcription-mode identities selected by global shortcut bindings.
//!
//! A binding selects the processing path before recording begins. Keeping the
//! mapping here avoids treating a failed local transcription as language
//! detection and gives future cloud modes an explicit, persisted route.

pub(crate) const TRANSCRIBE_BINDING_ID: &str = "transcribe";
pub(crate) const TRANSCRIBE_WITH_POST_PROCESS_BINDING_ID: &str = "transcribe_with_post_process";
pub(crate) const TRANSCRIBE_BANGLA_ROMANIZED_BINDING_ID: &str = "transcribe_bangla_romanized";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum TranscriptionMode {
    Local,
    LocalWithPostProcessing,
    BanglaRomanization,
}

impl TranscriptionMode {
    pub(crate) fn from_binding_id(binding_id: &str) -> Option<Self> {
        match binding_id {
            TRANSCRIBE_BINDING_ID => Some(Self::Local),
            TRANSCRIBE_WITH_POST_PROCESS_BINDING_ID => Some(Self::LocalWithPostProcessing),
            TRANSCRIBE_BANGLA_ROMANIZED_BINDING_ID => Some(Self::BanglaRomanization),
            _ => None,
        }
    }

    pub(crate) fn uses_local_inference(self) -> bool {
        !matches!(self, Self::BanglaRomanization)
    }

    pub(crate) fn requests_post_processing(self) -> bool {
        matches!(self, Self::LocalWithPostProcessing)
    }
}

pub(crate) fn is_transcription_binding(binding_id: &str) -> bool {
    TranscriptionMode::from_binding_id(binding_id).is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_transcription_binding_maps_to_one_mode() {
        assert_eq!(
            TranscriptionMode::from_binding_id(TRANSCRIBE_BINDING_ID),
            Some(TranscriptionMode::Local)
        );
        assert_eq!(
            TranscriptionMode::from_binding_id(TRANSCRIBE_WITH_POST_PROCESS_BINDING_ID),
            Some(TranscriptionMode::LocalWithPostProcessing)
        );
        assert_eq!(
            TranscriptionMode::from_binding_id(TRANSCRIBE_BANGLA_ROMANIZED_BINDING_ID),
            Some(TranscriptionMode::BanglaRomanization)
        );
        assert_eq!(TranscriptionMode::from_binding_id("cancel"), None);
    }

    #[test]
    fn only_local_modes_use_local_inference() {
        assert!(TranscriptionMode::Local.uses_local_inference());
        assert!(TranscriptionMode::LocalWithPostProcessing.uses_local_inference());
        assert!(!TranscriptionMode::BanglaRomanization.uses_local_inference());
        assert!(TranscriptionMode::LocalWithPostProcessing.requests_post_processing());
        assert!(!TranscriptionMode::BanglaRomanization.requests_post_processing());
    }
}
