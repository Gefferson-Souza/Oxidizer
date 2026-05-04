pub(crate) mod banner;
// Why allowed: colors module exposes a complete palette; some constants are
// reserved for upcoming banner/diagnostic surfaces (severity tiers, branding).
#[allow(dead_code)]
pub(crate) mod colors;
pub(crate) mod progress;
