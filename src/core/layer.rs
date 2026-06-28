//! Global stacking layers for the library's overlays.

/// Typed `GlobalZIndex` bands for library overlays. Higher = in front.
///
/// Centralizes what used to be scattered magic numbers (500/800/900/999) so the
/// layering order is defined in one place. Floating windows occupy a band
/// starting at [`ZLayer::Windows`] and growing upward per open window, staying
/// below the overlays above.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZLayer {
    /// Floating windows (base of the per-window band).
    Windows,
    /// Hover tooltips.
    Tooltip,
    /// Dialogue boxes.
    Dialogue,
    /// Modal dialogs + backdrop.
    Modal,
    /// Drag ghost — always on top.
    DragGhost,
}

impl ZLayer {
    /// The `GlobalZIndex` value for this layer.
    pub const fn z(self) -> i32 {
        match self {
            ZLayer::Windows => 10,
            ZLayer::Tooltip => 500,
            ZLayer::Dialogue => 800,
            ZLayer::Modal => 900,
            ZLayer::DragGhost => 999,
        }
    }
}
