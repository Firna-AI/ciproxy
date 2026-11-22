//! The released images that the runtime offers, at most one for each platform.

use std::collections::BTreeMap;
use std::sync::Arc;

use ciproxy_protocol::images::ImageDownload;
use ciproxy_protocol::platform::Platform;

use crate::image_signing::ImageSigning;

/// One released image that the runtime offers to the machines of its platform.
#[derive(Clone)]
pub struct ImageOffer {
    /// Release facts with the unsigned artifact URL.
    pub download: ImageDownload,
    /// Signs a fresh URL for each send. Without it, the URL goes out as it is.
    pub signing: Option<Arc<dyn ImageSigning>>,
}

impl ImageOffer {
    /// Offer a release whose URL needs no signature.
    pub fn unsigned(download: ImageDownload) -> Self {
        Self {
            download,
            signing: None,
        }
    }
}

/// Offers keyed by the platform of their own release.
///
/// A later offer of a platform replaces the earlier one, so the set never
/// holds two releases of one platform.
#[derive(Clone, Default)]
pub struct ImageOffers(BTreeMap<Platform, ImageOffer>);

impl ImageOffers {
    /// The offer that the machines of this platform get.
    pub fn get(&self, platform: Platform) -> Option<&ImageOffer> {
        self.0.get(&platform)
    }
}

impl FromIterator<ImageOffer> for ImageOffers {
    fn from_iter<T: IntoIterator<Item = ImageOffer>>(offers: T) -> Self {
        Self(
            offers
                .into_iter()
                .map(|offer| (offer.download.platform, offer))
                .collect(),
        )
    }
}
