use super::{
    BufferEditedEvent, EditorCore, EditorCoreEvent, MarkdownPreviewSnapshot,
    SearchFocusChangedEvent, SearchResultsUpdatedEvent,
};
use crate::editor::markdown::{MarkdownPreviewSyncedEvent, MarkdownServiceEvent};
use crate::editor::minimap::MinimapServiceEvent;
use crate::editor::smart_gutter::SmartGutterServiceEvent;
use crate::search::service::{SearchMatch, SearchQuery};
use std::path::{Path, PathBuf};

impl EditorCore {
    pub(super) fn push_search_results_event(
        &mut self,
        query: SearchQuery,
        matches: Vec<SearchMatch>,
    ) {
        let focus_index = self.search_navigator.current_index();
        let focus_match = self.search_navigator.current().cloned();
        self.events.push_back(EditorCoreEvent::SearchResultsUpdated(
            SearchResultsUpdatedEvent {
                query,
                matches,
                focus_index,
                focus_match,
            },
        ));
    }

    pub(super) fn push_search_focus_event(&mut self) {
        let focus_index = self.search_navigator.current_index();
        let focus_match = self.search_navigator.current().cloned();
        self.events.push_back(EditorCoreEvent::SearchFocusChanged(
            SearchFocusChangedEvent {
                focus_index,
                focus_match,
            },
        ));
    }

    pub(super) fn push_buffer_edited_event(
        &mut self,
        file_path: PathBuf,
        revision: u64,
        cursor_char: usize,
        is_dirty: bool,
    ) {
        self.events
            .push_back(EditorCoreEvent::BufferEdited(BufferEditedEvent {
                file_path,
                revision,
                cursor_char,
                is_dirty,
            }));
    }

    pub(super) fn push_markdown_observation_events(
        &mut self,
        file_path: &Path,
        revision: u64,
        previous_content: &str,
        current_content: &str,
    ) {
        let events = self.markdown_service.observe_change(
            file_path,
            revision,
            previous_content,
            current_content,
        );
        for event in events {
            self.push_markdown_service_event(event);
        }
    }

    pub(super) fn push_markdown_service_event(&mut self, event: MarkdownServiceEvent) {
        match event {
            MarkdownServiceEvent::FeatureRequested(event) => self
                .events
                .push_back(EditorCoreEvent::MarkdownFeatureRequested(event)),
            MarkdownServiceEvent::DiffObserved(event) => self
                .events
                .push_back(EditorCoreEvent::MarkdownDiffObserved(event)),
            MarkdownServiceEvent::PreviewSynced(event) => {
                self.update_markdown_preview_snapshot(&event);
                self.events
                    .push_back(EditorCoreEvent::MarkdownPreviewSynced(event));
            }
        }
    }

    fn update_markdown_preview_snapshot(&mut self, event: &MarkdownPreviewSyncedEvent) {
        self.markdown_preview_snapshot = Some(MarkdownPreviewSnapshot {
            file_path: event.file_path.clone(),
            revision: event.revision,
            headings: event.headings.clone(),
        });
    }

    pub(super) fn sync_services_after_buffer_update(&mut self, current_content: &str) {
        if let Some(event) = self.minimap_service.on_buffer_updated(current_content) {
            self.push_minimap_service_event(event);
        }
        if let Some(event) = self.smart_gutter_service.on_buffer_updated(current_content) {
            self.push_smart_gutter_service_event(event);
        }
    }

    pub(super) fn push_minimap_service_event(&mut self, event: MinimapServiceEvent) {
        match event {
            MinimapServiceEvent::OverlaysUpdated(event) => self
                .events
                .push_back(EditorCoreEvent::MinimapOverlaysUpdated(event)),
            MinimapServiceEvent::FocusIdSynced(event) => self
                .events
                .push_back(EditorCoreEvent::MinimapFocusIdSynced(event)),
        }
    }

    pub(super) fn push_smart_gutter_service_event(&mut self, event: SmartGutterServiceEvent) {
        match event {
            SmartGutterServiceEvent::IndicatorsUpdated(event) => self
                .events
                .push_back(EditorCoreEvent::SmartGutterIndicatorsUpdated(event)),
            SmartGutterServiceEvent::FocusIdSynced(event) => self
                .events
                .push_back(EditorCoreEvent::SmartGutterFocusIdSynced(event)),
            SmartGutterServiceEvent::JumpRequested(event) => self
                .events
                .push_back(EditorCoreEvent::SmartGutterJumpRequested(event)),
            SmartGutterServiceEvent::ApprovalRequestOpened(event) => self
                .events
                .push_back(EditorCoreEvent::SmartGutterApprovalRequestOpened(event)),
        }
    }
}
