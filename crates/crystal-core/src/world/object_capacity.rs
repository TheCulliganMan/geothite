//! Object capacity for original maps and larger authored maps.

/// Maximum non-player map records and live slots supported by byte-sized IDs.
/// Slot zero belongs to the player. Authored maps may use every remaining ID.
pub const MAX_MAP_OBJECTS: usize = u8::MAX as usize;

/// Keep the original minimum allocation, without truncating maps with thirteen
/// to fifteen residents. Object identities remain byte-sized, including the
/// reserved player slot. Offscreen loading and unloading rules are unchanged.
pub(crate) fn loaded_object_capacity(map_object_count: usize) -> usize {
    map_object_count.clamp(12, MAX_MAP_OBJECTS)
}
