//! Session-only visual placement drafts and bounded controller overrides.

use super::*;
use crate::commands::{HostSnapshot, InstanceState, VisualPlacement};

/// A destination/session/catalog boundary; overrides never cross this identity.
#[derive(Clone, Debug, PartialEq, Eq)]
pub(super) struct PlacementScope {
    /// Distinguish a local host from a remote host with identical public IDs.
    pub(super) remote: bool,
    /// Authoritative host lifetime.
    pub(super) session: String,
    /// Pinned catalog content.
    pub(super) revision: String,
}

impl PlacementScope {
    /// Capture the currently selected authoritative destination, if connected.
    pub(super) fn current(
        runtime: &Runtime,
        network: &crate::network_app::NetworkState,
    ) -> Option<Self> {
        let remote = runtime.destination == Destination::Remote;
        let snapshot = if remote {
            if !network.connected {
                return None;
            }
            network.snapshot.as_ref()?
        } else {
            &runtime.snapshot
        };
        Some(Self::from_snapshot(remote, snapshot))
    }

    /// Bind a controller draft to one snapshot identity.
    pub(super) fn from_snapshot(remote: bool, snapshot: &HostSnapshot) -> Self {
        Self {
            remote,
            session: snapshot.session_id.clone(),
            revision: snapshot.catalog_revision.clone(),
        }
    }
}

/// Isolated editable percentages; canceling drops every draft change, including reset.
pub(super) struct PlacementModal {
    /// Exact clip selected by the Edit control.
    pub(super) clip_id: String,
    /// Human-readable title shown explicitly in the dialog heading.
    pub(super) title: String,
    /// Immutable host/catalog default, retained for Reset.
    pub(super) defaults: VisualPlacement,
    /// Three ordinary Unicode editors: width, horizontal and vertical percentages.
    pub(super) editors: [Editor; 3],
    /// Validation feedback for the current attempted apply.
    pub(super) error: Option<String>,
    /// Independent bounded form scroll position.
    pub(super) scroll: f32,
    /// Reset applies the catalog default by clearing the occurrence override.
    pub(super) reset: bool,
}

impl PlacementModal {
    /// Initialize editable values from a validated placement.
    pub(super) fn new(
        clip_id: String,
        title: String,
        defaults: VisualPlacement,
        initial: VisualPlacement,
    ) -> Self {
        Self {
            clip_id,
            title,
            defaults,
            editors: Self::editors(initial),
            error: None,
            scroll: 0.,
            reset: false,
        }
    }

    /// Display basis points as exact percentages without floating point drift.
    pub(super) fn editors(value: VisualPlacement) -> [Editor; 3] {
        [value.width_bps, value.x_bps, value.y_bps].map(|bps| {
            let text = format!("{}.{:02}", bps / 100, bps % 100);
            Editor { text, ..default() }
        })
    }

    /// Restore catalog values in the draft only; Apply or Send commits this reset.
    pub(super) fn reset(&mut self) {
        self.editors = Self::editors(self.defaults);
        self.reset = true;
        self.error = None;
    }

    /// Nudge a valid percentage by one point, clamping to the same bounds as text input.
    pub(super) fn step(&mut self, index: usize, direction: i16) {
        let Some(editor) = self.editors.get_mut(index) else {
            return;
        };
        let Some(current) = Self::percentage(&editor.text, index == 0) else {
            self.error = Some("Enter a valid percentage before using − or +.".into());
            return;
        };
        let value = (i32::from(current) + i32::from(direction) * 100)
            .clamp(if index == 0 { 1 } else { 0 }, 10_000) as u16;
        let text = format!("{}.{:02}", value / 100, value % 100);
        *editor = Editor { text, ..default() };
        self.reset = false;
        self.error = None;
    }

    /// Parse a finite, unsigned decimal percentage with at most two decimal places.
    pub(super) fn percentage(text: &str, positive: bool) -> Option<u16> {
        let text = text.trim();
        let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
        if whole.is_empty()
            || !whole.bytes().all(|c| c.is_ascii_digit())
            || fraction.len() > 2
            || !fraction.bytes().all(|c| c.is_ascii_digit())
        {
            return None;
        }
        let whole: u16 = whole.parse().ok()?;
        let fraction: u16 = if fraction.is_empty() {
            0
        } else {
            fraction.parse().ok()?
        };
        let bps = whole.checked_mul(100)?.checked_add(
            if text.split_once('.').is_some_and(|(_, f)| f.len() == 1) {
                fraction * 10
            } else {
                fraction
            },
        )?;
        (bps <= 10_000 && (!positive || bps > 0)).then_some(bps)
    }

    /// Validate every field before any active occurrence or remembered value changes.
    pub(super) fn placement(&self) -> Result<VisualPlacement, &'static str> {
        let width_bps = Self::percentage(&self.editors[0].text, true).ok_or(
            "Scale must be greater than 0 and at most 100%, with up to two decimal places.",
        )?;
        let x_bps = Self::percentage(&self.editors[1].text, false)
            .ok_or("Horizontal position must be 0–100%, with up to two decimal places.")?;
        let y_bps = Self::percentage(&self.editors[2].text, false)
            .ok_or("Vertical position must be 0–100%, with up to two decimal places.")?;
        Ok(VisualPlacement {
            width_bps,
            x_bps,
            y_bps,
        })
    }
}

impl Model {
    /// Close stale drafts and forget overrides whenever destination, lifetime or catalog changes.
    pub(super) fn sync_placements(&mut self, scope: Option<PlacementScope>) {
        if self.placement_scope != scope {
            self.placements.clear();
            self.placement_order.clear();
            self.placement_modal = None;
            if matches!(self.focus, Some(Field::Placement(_))) {
                self.focus = None;
            }
            self.placement_scope = scope;
            self.revision = self.revision.wrapping_add(1);
        }
    }

    /// Use session adjustments for all subsequent panel plays of the selected clip.
    pub(super) fn play_action(&self, clip_id: &str) -> Action {
        if let Some(placement) = self.placements.get(clip_id) {
            Action::PlayVisual {
                clip_id: clip_id.into(),
                placement: *placement,
            }
        } else {
            Action::Play(clip_id.into())
        }
    }

    /// Open only visual clips, resolving defaults from the selected host's real projection.
    pub(super) fn open_placement(
        &mut self,
        clip_id: &str,
        runtime: &Runtime,
        network: &crate::network_app::NetworkState,
    ) {
        let (title, defaults, snapshot) = if runtime.destination == Destination::Remote {
            let Some(clip) = network
                .remote_catalog
                .as_ref()
                .and_then(|catalog| catalog.clips.iter().find(|clip| clip.clip_id == clip_id))
            else {
                return;
            };
            let Some(defaults) = clip.default_placement else {
                return;
            };
            let Some(snapshot) = network.snapshot.as_ref() else {
                return;
            };
            (clip.title.clone(), defaults, snapshot)
        } else {
            let Some(clip) = runtime
                .catalog
                .clips
                .iter()
                .find(|clip| clip.identity == clip_id)
            else {
                return;
            };
            let Some(defaults) = crate::host::Host::default_placement(clip) else {
                return;
            };
            (clip.metadata.title.clone(), defaults, &runtime.snapshot)
        };
        let active = snapshot.instances.iter().rev().find(|instance| {
            instance.clip_id == clip_id
                && matches!(
                    instance.state,
                    InstanceState::Preparing | InstanceState::Playing
                )
        });
        let initial = active
            .map(|instance| instance.placement.unwrap_or(defaults))
            .or_else(|| self.placements.get(clip_id).copied())
            .unwrap_or(defaults);
        self.close();
        self.placement_modal = Some(PlacementModal::new(
            clip_id.into(),
            title,
            defaults,
            initial,
        ));
        self.dragging = false;
    }

    /// Atomically remember a validated draft, update all live occurrences, and optionally play.
    pub(super) fn apply_placement(&mut self, snapshot: &HostSnapshot, send: bool) -> Vec<Action> {
        let Some(modal) = &mut self.placement_modal else {
            return Vec::new();
        };
        let placement = match modal.placement() {
            Ok(placement) => placement,
            Err(error) => {
                modal.error = Some(error.into());
                return Vec::new();
            }
        };
        let clip_id = modal.clip_id.clone();
        let placement = if modal.reset && placement == modal.defaults {
            None
        } else {
            Some(placement)
        };
        if let Some(placement) = placement {
            if self.placements.len() >= 256
                && !self.placements.contains_key(&clip_id)
                && let Some(oldest) = self.placement_order.pop_front()
            {
                self.placements.remove(&oldest);
            }
            self.placements.insert(clip_id.clone(), placement);
            self.placement_order.retain(|id| id != &clip_id);
            self.placement_order.push_back(clip_id.clone());
        } else {
            self.placements.remove(&clip_id);
            self.placement_order.retain(|id| id != &clip_id);
        }
        let mut actions: Vec<_> = snapshot
            .instances
            .iter()
            .filter(|instance| {
                instance.clip_id == clip_id
                    && matches!(
                        instance.state,
                        InstanceState::Preparing | InstanceState::Playing
                    )
            })
            .map(|instance| Action::SetVisual {
                instance_id: instance.instance_id.clone(),
                placement,
            })
            .collect();
        if send {
            actions.push(self.play_action(&clip_id));
        }
        self.close();
        actions
    }
}
