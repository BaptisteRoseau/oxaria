//! WCAG 1.2.2 / 2.2.2 checks for `video` elements.

use crate::page::{ElementRef, RenderedPage};

use super::{CheckOptions, Finding};

/// G87: a prerecorded video needs a captions (or subtitles) track for deaf and hard-of-hearing
/// users to follow its audio.
pub fn check_video_missing_captions(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.by_tag("video")
        .filter(|video| !has_captions_track(*video))
        .map(|_| {
            Finding::error(
                "G87",
                "<video> has no captions or subtitles track".to_string(),
            )
        })
        .collect()
}

fn has_captions_track(video: ElementRef) -> bool {
    video
        .descendants()
        .filter(|el| el.tag() == "track")
        .any(|track| matches!(track.attr("kind"), Some("captions") | Some("subtitles")))
}

/// G152/F16: an autoplaying video without visible controls gives users no way to pause or stop
/// it, which WCAG 2.2.2 requires for content that plays automatically.
pub fn check_autoplay_without_controls(
    page: &RenderedPage,
    _options: &CheckOptions,
) -> Vec<Finding> {
    page.select(|el| el.tag() == "video" && el.has_attr("autoplay"))
        .into_iter()
        .filter(|video| video.attr("controls").is_none())
        .map(|_| {
            Finding::error(
                "G152",
                "<video autoplay> has no controls to pause it".to_string(),
            )
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::page::testutil::page_from_html;

    fn options() -> CheckOptions {
        CheckOptions {
            contrast_threshold: 4.5,
            large_text_contrast_threshold: 3.0,
            target_size_threshold: 24.0,
        }
    }

    #[test]
    fn video_without_captions_is_flagged() {
        let p = page_from_html(r#"<video src="a.mp4" controls></video>"#);
        let findings = check_video_missing_captions(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "G87");
    }

    #[test]
    fn video_with_captions_track_is_not_flagged() {
        let p = page_from_html(
            r#"<video src="a.mp4" controls><track kind="captions" src="a.vtt"></video>"#,
        );
        assert!(check_video_missing_captions(&p, &options()).is_empty());
    }

    #[test]
    fn autoplay_without_controls_is_flagged() {
        let p = page_from_html(r#"<video src="a.mp4" autoplay></video>"#);
        let findings = check_autoplay_without_controls(&p, &options());
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].rule_id, "G152");
    }

    #[test]
    fn autoplay_with_controls_is_not_flagged() {
        let p = page_from_html(r#"<video src="a.mp4" autoplay controls></video>"#);
        assert!(check_autoplay_without_controls(&p, &options()).is_empty());
    }

    #[test]
    fn non_autoplay_video_is_not_flagged() {
        let p = page_from_html(r#"<video src="a.mp4"></video>"#);
        assert!(check_autoplay_without_controls(&p, &options()).is_empty());
    }
}
