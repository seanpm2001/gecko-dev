use super::{InitTracker, MemoryInitKind};
use crate::{id::TextureId, track::TextureSelector};
use arrayvec::ArrayVec;
use std::ops::Range;

#[derive(Debug, Clone)]
pub(crate) struct TextureInitRange {
    pub(crate) mip_range: Range<u32>,
    pub(crate) layer_range: Range<u32>,
}

impl From<TextureSelector> for TextureInitRange {
    fn from(selector: TextureSelector) -> Self {
        TextureInitRange {
            mip_range: selector.levels,
            layer_range: selector.layers,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct TextureInitTrackerAction {
    pub(crate) id: TextureId,
    pub(crate) range: TextureInitRange,
    pub(crate) kind: MemoryInitKind,
}

pub(crate) type TextureLayerInitTracker = InitTracker<u32>;

#[derive(Debug)]
pub(crate) struct TextureInitTracker {
    pub mips: ArrayVec<TextureLayerInitTracker, { hal::MAX_MIP_LEVELS as usize }>,
}

impl TextureInitTracker {
    pub(crate) fn new(mip_level_count: u32, depth_or_array_layers: u32) -> Self {
        TextureInitTracker {
            mips: std::iter::repeat(TextureLayerInitTracker::new(depth_or_array_layers))
                .take(mip_level_count as usize)
                .collect(),
        }
    }

    pub(crate) fn check_action(
        &self,
        action: &TextureInitTrackerAction,
    ) -> Option<TextureInitTrackerAction> {
        let mut mip_range_start = std::usize::MAX;
        let mut mip_range_end = std::usize::MIN;
        let mut layer_range_start = std::u32::MAX;
        let mut layer_range_end = std::u32::MIN;

        for (i, mip_tracker) in self
            .mips
            .iter()
            .enumerate()
            .take(action.range.mip_range.end as usize)
            .skip(action.range.mip_range.start as usize)
        {
            if let Some(uninitialized_layer_range) =
                mip_tracker.check(action.range.layer_range.clone())
            {
                mip_range_start = mip_range_start.min(i);
                mip_range_end = i + 1;
                layer_range_start = layer_range_start.min(uninitialized_layer_range.start);
                layer_range_end = layer_range_end.max(uninitialized_layer_range.end);
            };
        }

        if mip_range_start < mip_range_end && layer_range_start < layer_range_end {
            Some(TextureInitTrackerAction {
                id: action.id,
                range: TextureInitRange {
                    mip_range: mip_range_start as u32..mip_range_end as u32,
                    layer_range: layer_range_start..layer_range_end,
                },
                kind: action.kind,
            })
        } else {
            None
        }
    }

    pub(crate) fn discard(&mut self, mip_level: u32, layer: u32) {
        self.mips[mip_level as usize].discard(layer);
    }
}
