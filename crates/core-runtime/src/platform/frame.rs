use crate::platform::scene::SceneSnapshot;
use std::sync::Arc;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameTicket {
    surface_generation: u64,
    frame_id: u64,
    scene_revision: u64,
}

impl FrameTicket {
    pub fn surface_generation(self) -> u64 {
        self.surface_generation
    }

    pub fn frame_id(self) -> u64 {
        self.frame_id
    }

    pub fn scene_revision(self) -> u64 {
        self.scene_revision
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FramePhase {
    Idle,
    Preparing(FrameTicket),
    Rendering(FrameTicket),
    Presented(FrameTicket),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FrameError {
    InvalidGeneration,
    Busy,
    StaleTicket,
    InvalidTransition,
    IdentifierExhausted,
}

#[derive(Default)]
pub struct FrameTracker {
    next_frame_id: u64,
    phase: FramePhase,
    active_scene: Option<Arc<SceneSnapshot>>,
}

impl Default for FramePhase {
    fn default() -> Self {
        Self::Idle
    }
}

impl FrameTracker {
    pub fn phase(&self) -> FramePhase {
        self.phase
    }

    pub fn begin(
        &mut self,
        surface_generation: u64,
        scene: Arc<SceneSnapshot>,
    ) -> Result<FrameTicket, FrameError> {
        if surface_generation == 0 {
            return Err(FrameError::InvalidGeneration);
        }
        if matches!(
            self.phase,
            FramePhase::Preparing(_) | FramePhase::Rendering(_)
        ) {
            let active = match self.phase {
                FramePhase::Preparing(ticket) | FramePhase::Rendering(ticket) => ticket,
                FramePhase::Idle | FramePhase::Presented(_) => unreachable!(),
            };
            if active.surface_generation == surface_generation {
                return Err(FrameError::Busy);
            }
            self.phase = FramePhase::Idle;
            self.active_scene = None;
        }
        let frame_id = self
            .next_frame_id
            .checked_add(1)
            .ok_or(FrameError::IdentifierExhausted)?;
        let ticket = FrameTicket {
            surface_generation,
            frame_id,
            scene_revision: scene.layout_revision(),
        };
        self.next_frame_id = frame_id;
        self.phase = FramePhase::Preparing(ticket);
        self.active_scene = Some(scene);
        Ok(ticket)
    }

    pub fn scene(&self, ticket: FrameTicket) -> Result<Arc<SceneSnapshot>, FrameError> {
        if !matches!(self.phase, FramePhase::Preparing(active) | FramePhase::Rendering(active) if active == ticket)
        {
            return Err(FrameError::StaleTicket);
        }
        self.active_scene
            .as_ref()
            .cloned()
            .ok_or(FrameError::StaleTicket)
    }

    pub fn mark_submitted(&mut self, ticket: FrameTicket) -> Result<(), FrameError> {
        match self.phase {
            FramePhase::Preparing(active) if active == ticket => {
                self.phase = FramePhase::Rendering(ticket);
                Ok(())
            }
            FramePhase::Preparing(_) | FramePhase::Rendering(_) | FramePhase::Presented(_) => {
                Err(FrameError::StaleTicket)
            }
            FramePhase::Idle => Err(FrameError::InvalidTransition),
        }
    }

    pub fn complete(&mut self, ticket: FrameTicket) -> bool {
        if self.phase != FramePhase::Rendering(ticket) {
            return false;
        }
        self.phase = FramePhase::Presented(ticket);
        self.active_scene = None;
        true
    }

    pub fn fail(&mut self, ticket: FrameTicket) -> bool {
        if !matches!(self.phase, FramePhase::Preparing(active) | FramePhase::Rendering(active) if active == ticket)
        {
            return false;
        }
        self.phase = FramePhase::Idle;
        self.active_scene = None;
        true
    }

    pub fn reset(&mut self) {
        self.phase = FramePhase::Idle;
        self.active_scene = None;
    }
}

#[cfg(test)]
mod tests {
    use super::{FrameError, FramePhase, FrameTicket, FrameTracker};
    use crate::platform::scene::{Damage, DeviceScale, SceneSnapshot};
    use std::sync::Arc;

    fn scene(revision: u64) -> Arc<SceneSnapshot> {
        Arc::new(
            SceneSnapshot::new(
                revision,
                DeviceScale::new(1.0, 1.0).expect("valid test scale"),
                [0.0, 0.0, 0.0],
                Arc::from([]),
                Damage::Full,
            )
            .expect("empty immutable scene"),
        )
    }

    #[test]
    fn platform_frame_submit_does_not_present_and_matching_completion_presents() {
        let mut frames = FrameTracker::default();
        let ticket = frames.begin(3, scene(7)).expect("begin frame");
        assert_eq!(frames.phase(), FramePhase::Preparing(ticket));
        assert_eq!(
            frames
                .scene(ticket)
                .expect("active snapshot")
                .layout_revision(),
            7
        );
        frames.mark_submitted(ticket).expect("submit frame");
        assert_eq!(frames.phase(), FramePhase::Rendering(ticket));

        let stale = FrameTicket {
            surface_generation: 4,
            ..ticket
        };
        assert!(!frames.complete(stale));
        assert_eq!(frames.begin(3, scene(8)), Err(FrameError::Busy));
        assert_eq!(frames.phase(), FramePhase::Rendering(ticket));
        assert!(frames.complete(ticket));
        assert_eq!(frames.phase(), FramePhase::Presented(ticket));
        assert_eq!(frames.scene(ticket), Err(FrameError::StaleTicket));
    }

    #[test]
    fn platform_frame_failure_allows_retry_and_ignores_old_generation_completion() {
        let mut frames = FrameTracker::default();
        let old = frames.begin(1, scene(2)).expect("old generation frame");
        frames.mark_submitted(old).expect("submit old frame");

        let current = frames
            .begin(2, scene(3))
            .expect("new generation replaces old");
        assert_ne!(old.frame_id(), current.frame_id());
        assert_eq!(frames.scene(old), Err(FrameError::StaleTicket));
        assert!(!frames.complete(old));
        assert_eq!(
            frames
                .scene(current)
                .expect("current snapshot")
                .layout_revision(),
            3
        );
        assert!(frames.fail(current));
        assert_eq!(frames.phase(), FramePhase::Idle);

        let retry = frames
            .begin(2, scene(3))
            .expect("retry same scene revision");
        assert_ne!(current.frame_id(), retry.frame_id());
        assert!(frames.mark_submitted(retry).is_ok());
        assert!(frames.complete(retry));
    }

    #[test]
    fn platform_frame_invalid_generation_and_out_of_order_submission_are_rejected() {
        let mut frames = FrameTracker::default();
        assert_eq!(
            frames.begin(0, scene(0)),
            Err(FrameError::InvalidGeneration)
        );
        let ticket = frames.begin(1, scene(1)).expect("begin frame");
        assert_eq!(frames.mark_submitted(ticket), Ok(()));
        assert_eq!(frames.mark_submitted(ticket), Err(FrameError::StaleTicket));
    }
}
