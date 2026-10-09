//! WCAG 1.2.2 / 1.4.2 / 2.2.2 checks for media and moving content.

use crate::page::{ElementRef, RenderedPage};

use crate::rules::Finding;

/// G87: a prerecorded video needs a captions (or subtitles) track for deaf and hard-of-hearing
/// users to follow its audio.
pub fn check_video_missing_captions(page: &RenderedPage) -> Vec<Finding> {
    page.by_tag("video")
        .filter(|video| !has_captions_track(*video))
        .map(|video| {
            Finding::error(
                "G87",
                "<video> has no captions or subtitles track".to_string(),
            )
            .at(video)
            .help(
                "add <track kind=\"captions\" src=\"captions.vtt\" srclang=\"en\" \
                 label=\"English\"> inside it; a video without audio needs none",
            )
        })
        .collect()
}

fn has_captions_track(video: ElementRef) -> bool {
    video
        .descendants()
        .filter(|el| el.tag() == "track")
        .any(|track| matches!(track.attr("kind"), Some("captions" | "subtitles")))
}

/// G186: an autoplaying video without visible controls gives users no way to pause or stop
/// it, which WCAG 2.2.2 requires for content that plays automatically.
pub fn check_autoplay_without_controls(page: &RenderedPage) -> Vec<Finding> {
    page.all()
        .filter(|el| el.tag() == "video" && el.has_attr("autoplay"))
        .filter(|video| video.attr("controls").is_none())
        .map(|video| {
            Finding::error(
                "G186",
                "<video autoplay> has no controls to pause it".to_string(),
            )
            .at(video)
            .help("add the controls attribute, or remove autoplay (muted is not enough)")
        })
        .collect()
}

/// F93: an `audio` or `video` element that starts playing sound on its own, with no controls,
/// talks over screen readers with no way to stop it.
pub fn check_autoplay_audio_without_controls(page: &RenderedPage) -> Vec<Finding> {
    page.all()
        .filter(|el| matches!(el.tag(), "audio" | "video"))
        .filter(|media| media.has_attr("autoplay") && !media.has_attr("muted"))
        .filter(|media| !media.has_attr("controls"))
        .map(|media| {
            Finding::error(
                "F93",
                format!(
                    "<{} autoplay> plays sound with no controls to pause or stop it",
                    media.tag()
                ),
            )
            .at(media)
            .help("add the controls attribute, add muted, or remove autoplay")
        })
        .collect()
}

/// F16: a `marquee` scrolls for as long as the page is open, and offers no way to pause it.
pub fn check_marquee(page: &RenderedPage) -> Vec<Finding> {
    page.by_tag("marquee")
        .map(|marquee| {
            Finding::error(
                "F16",
                "<marquee> scrolls content with no way to pause it".to_string(),
            )
            .at(marquee)
            .help("show the text statically, or animate it with a visible pause button")
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::rules::testutil::{assert_fires, rule_ids, run};

    #[rstest]
    #[case(r#"<video src="a.mp4" controls></video>"#, &["G87"])]
    #[case(r#"<video src="a.mp4" controls><track kind="captions" src="a.vtt"></video>"#, &[])]
    fn video_missing_captions(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(
            rule_ids(check_video_missing_captions, html),
            expected,
            "{html}"
        );
    }

    #[rstest]
    #[case(r#"<video src="a.mp4" autoplay></video>"#, &["G186"])]
    #[case(r#"<video src="a.mp4" autoplay controls></video>"#, &[])]
    #[case(r#"<video src="a.mp4"></video>"#, &[])]
    fn autoplay_without_controls(#[case] html: &str, #[case] expected: &[&str]) {
        assert_eq!(
            rule_ids(check_autoplay_without_controls, html),
            expected,
            "{html}"
        );
    }

    #[test]
    fn unmuted_autoplay_without_controls_is_flagged() {
        let html = r#"<video src="ads.cgi?kind=video" autoplay loop></video><audio src="a.mp3" autoplay></audio>"#;
        let findings = run(check_autoplay_audio_without_controls, html);
        assert_eq!(findings.len(), 2);
        assert_eq!(findings[0].rule_id, "F93");
        assert!(findings[1].message.starts_with("<audio autoplay>"));
    }

    #[test]
    fn muted_or_controlled_autoplay_is_not_flagged_for_sound() {
        let html = r#"<video src="a.mp4" autoplay muted></video><audio src="a.mp3" autoplay controls></audio>"#;
        assert!(run(check_autoplay_audio_without_controls, html).is_empty());
    }

    #[test]
    fn marquee_is_flagged() {
        let html = "<marquee>Breaking news: sale ends today!</marquee>";
        assert_fires(check_marquee, html, "F16");
    }
}
