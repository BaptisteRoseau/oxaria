//! ACT rules for text alternatives of images, image buttons, SVG graphics
//! and embedded objects, and for elements marked decorative.

use super::name::accessible_name;
use crate::page::{ElementRef, RenderedPage};
use crate::rules::aria_spec::{
    Parsed, attribute, concrete_role, effective_role, explicit_roles, first_valid_role, input_type,
    is_focusable, is_hidden, parse_value,
};
use crate::rules::{CheckOptions, Finding};

const SVG_IMAGE_ROLES: &[&str] = &["img", "graphics-document", "graphics-symbol"];

/// The Graphics-ARIA roles, which WAI-ARIA 1.2's role table lacks.
const GRAPHICS_ROLES: &[&str] = &["graphics-document", "graphics-object", "graphics-symbol"];

const MEDIA_EXTENSIONS: &[&str] = &[
    "aac", "apng", "avi", "avif", "bmp", "flac", "gif", "ico", "jpeg", "jpg", "m4a", "m4v", "mkv",
    "mov", "mp3", "mp4", "mpeg", "mpg", "oga", "ogg", "ogv", "opus", "png", "svg", "tif", "tiff",
    "wav", "weba", "webm", "webp",
];

/// 23a2a8: every `img` element and element with the `img` role needs a
/// non-empty accessible name, unless it is exposed as `none`/`presentation`.
pub fn check_image_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_html_image(*el) && !is_hidden(*el))
        .filter(|img| !is_exposed_as_presentational(*img))
        .filter(|img| accessible_name(*img).is_empty())
        .map(|img| {
            Finding::error(
                "23a2a8",
                format!("{} has no accessible name", image_markup(img)),
            )
            .at(img)
            .help(image_name_help(img))
        })
        .collect()
}

fn is_html_image(el: ElementRef) -> bool {
    !is_svg_element(el)
        && (el.tag() == "img" || effective_role(el).is_some_and(|role| role.name == "img"))
}

fn is_exposed_as_presentational(el: ElementRef) -> bool {
    effective_role(el).is_some_and(|role| role.is_presentational())
}

fn image_markup(img: ElementRef) -> String {
    match (img.tag(), img.attr("src"), img.attr("alt")) {
        ("img", Some(src), Some(alt)) => format!("<img src=\"{src}\" alt=\"{alt}\">"),
        ("img", Some(src), None) => format!("<img src=\"{src}\">"),
        (tag, _, _) => format!("<{tag} role=\"{}\">", img.attr("role").unwrap_or("img")),
    }
}

fn image_name_help(img: ElementRef) -> &'static str {
    match img.tag() {
        "img" => {
            "add alt=\"...\" describing the image, or alt=\"\" (without other naming \
             attributes) if it is purely decorative"
        }
        _ => "add aria-label=\"...\" describing the image, or role=\"none\" if it is decorative",
    }
}

/// 46ca7f: an element marked decorative (`role="none"`/`"presentation"`, or
/// an `img` with `alt=""`) must not be exposed again by a global ARIA
/// attribute or by being focusable.
pub fn check_decorative_exposed(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_marked_decorative(*el) && !is_hidden(*el))
        .filter_map(|el| Some((el, exposure(el)?)))
        .map(|(el, reason)| {
            Finding::error(
                "46ca7f",
                format!(
                    "<{}{}> is marked decorative but {reason}",
                    el.tag(),
                    decorative_marking(el)
                ),
            )
            .at(el)
            .help(
                "remove the ARIA attributes and tabindex that expose it, or drop the decorative \
                 marking and give it a real name if it conveys information",
            )
        })
        .collect()
}

fn is_marked_decorative(el: ElementRef) -> bool {
    first_valid_role(el).is_some_and(|role| role.is_presentational())
        || (el.tag() == "img" && el.attr("alt") == Some(""))
}

fn decorative_marking(el: ElementRef) -> String {
    match el.attr("role") {
        Some(role) if first_valid_role(el).is_some_and(|r| r.is_presentational()) => {
            format!(" role=\"{role}\"")
        }
        _ => " alt=\"\"".to_string(),
    }
}

/// Why WAI-ARIA's presentational role conflict resolution exposes it.
fn exposure(el: ElementRef) -> Option<String> {
    let attributes = global_aria_attributes(el);
    match (attributes.is_empty(), is_focusable(el)) {
        (false, _) => Some(format!("is exposed by {}", attributes.join(", "))),
        (true, true) => Some("is exposed because it is focusable".to_string()),
        (true, false) => None,
    }
}

fn global_aria_attributes(el: ElementRef) -> Vec<String> {
    el.node()
        .attrs
        .iter()
        .filter(|(name, value)| {
            attribute(name).is_some_and(|attribute| {
                attribute.is_global() && parse_value(attribute, value) != Parsed::Empty
            })
        })
        .map(|(name, value)| format!("{name}=\"{value}\""))
        .collect()
}

/// 59796f: an `input type="image"` needs a name from `alt`, `aria-label`,
/// `aria-labelledby` or `title`; otherwise it is announced as "Submit Query".
pub fn check_image_button_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.by_tag("input")
        .filter(|input| input_type(*input) == "image" && !is_hidden(*input))
        .filter(|input| accessible_name(*input).is_empty())
        .map(|input| {
            let src = input.attr("src").unwrap_or("...");
            Finding::error(
                "59796f",
                format!("<input type=\"image\" src=\"{src}\"> has no accessible name"),
            )
            .at(input)
            .help(format!(
                "add alt=\"...\" naming the button's action, e.g. \
                 <input type=\"image\" src=\"{src}\" alt=\"Search\">; the name attribute doesn't count"
            ))
        })
        .collect()
}

/// 7d6734: an SVG element with an explicit `img`, `graphics-document` or
/// `graphics-symbol` role needs a non-empty accessible name.
pub fn check_svg_image_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.all()
        .filter(|el| is_svg_element(*el) && !is_hidden(*el))
        .filter_map(|el| Some((el, svg_image_role(el)?)))
        .filter(|(el, _)| accessible_name(*el).is_empty())
        .map(|(el, role)| {
            Finding::error(
                "7d6734",
                format!("<{} role=\"{role}\"> has no accessible name", el.tag()),
            )
            .at(el)
            .help(format!(
                "add a <title> as the first child of the <{}>, or aria-label=\"...\"",
                el.tag()
            ))
        })
        .collect()
}

fn is_svg_element(el: ElementRef) -> bool {
    el.tag() == "svg" || el.ancestors().any(|ancestor| ancestor.tag() == "svg")
}

/// The first `role` token naming a WAI-ARIA or Graphics-ARIA role, when it
/// is one of the image roles.
fn svg_image_role(el: ElementRef) -> Option<String> {
    let role = explicit_roles(el)
        .into_iter()
        .find(|token| concrete_role(token).is_some() || GRAPHICS_ROLES.contains(&token.as_str()))?;
    let canonical = concrete_role(&role).map_or(role.as_str(), |role| role.name);
    SVG_IMAGE_ROLES.contains(&canonical).then_some(role)
}

/// 8fc3b6: an `object` without a role that embeds an image, audio or video
/// needs a non-empty accessible name.
pub fn check_object_name(page: &RenderedPage, _options: &CheckOptions) -> Vec<Finding> {
    page.by_tag("object")
        .filter(|object| first_valid_role(*object).is_none() && !is_hidden(*object))
        .filter(|object| embeds_media(*object))
        .filter(|object| accessible_name(*object).is_empty())
        .map(|object| {
            let data = object.attr("data").unwrap_or("...");
            Finding::error(
                "8fc3b6",
                format!("<object data=\"{data}\"> has no accessible name"),
            )
            .at(object)
            .help(
                "add title=\"...\" or aria-label=\"...\" describing the media; \
                 alt and fallback content inside the <object> don't name it",
            )
        })
        .collect()
}

/// litehtml loads nothing, so the MIME type comes from the `type`
/// attribute or, failing that, the `data` URL's extension.
fn embeds_media(object: ElementRef) -> bool {
    match object.attr("type").map(|t| t.trim().to_ascii_lowercase()) {
        Some(mime) if !mime.is_empty() => ["image/", "audio/", "video/"]
            .iter()
            .any(|kind| mime.starts_with(kind)),
        _ => object.attr("data").is_some_and(has_media_extension),
    }
}

fn has_media_extension(url: &str) -> bool {
    let path = url.split(['?', '#']).next().unwrap_or(url);
    let file = path.rsplit('/').next().unwrap_or(path);
    file.rsplit_once('.').is_some_and(|(_, extension)| {
        MEDIA_EXTENSIONS.contains(&extension.to_ascii_lowercase().as_str())
    })
}

#[cfg(test)]
mod tests {
    use rstest::rstest;

    use super::*;
    use crate::page::testutil::page_from_html;
    use crate::rules::RuleCheck;

    fn count(check: RuleCheck, html: &str) -> usize {
        check(&page_from_html(html), &CheckOptions::default()).len()
    }

    #[rstest]
    #[case(r#"<img src="w3c.png">"#)]
    #[case(r#"<div role="img" style="width:72px; height:48px"></div>"#)]
    #[case(r#"<div style="margin-left:-9999px"><img src="w3c.png"></div>"#)]
    #[case(r#"<img src="w3c.png" alt=" ">"#)]
    #[case(r#"<img role="none" tabindex="0" src="w3c.png">"#)]
    #[case(r#"<div role="image">text is not a name</div>"#)]
    fn unnamed_images_fail_23a2a8(#[case] html: &str) {
        assert_eq!(count(check_image_name, html), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<img alt="W3C logo" src="w3c.png">"#)]
    #[case(r#"<img alt="" src="background.png">"#)]
    #[case(r#"<div role="img" aria-label="W3C logo"></div>"#)]
    #[case(r#"<div id="l" hidden>W3C logo</div><div role="img" aria-labelledby="l"></div>"#)]
    #[case(r#"<img title="W3C logo" src="w3c.png">"#)]
    #[case(r#"<img role="presentation" src="w3c.png">"#)]
    #[case(r#"<img role="none" src="w3c.png">"#)]
    #[case(r#"<img src="w3c.png" aria-hidden="true">"#)]
    #[case(r#"<div hidden><img src="w3c.png"></div>"#)]
    #[case(r#"<svg role="img"></svg>"#)]
    fn named_decorative_or_hidden_images_pass_23a2a8(#[case] html: &str) {
        assert_eq!(count(check_image_name, html), 0, "{html}");
    }

    #[test]
    fn image_message_quotes_the_markup() {
        let page = page_from_html(r#"<img src="w3c.png">"#);
        let findings = check_image_name(&page, &CheckOptions::default());
        assert_eq!(
            findings[0].message,
            r#"<img src="w3c.png"> has no accessible name"#
        );
    }

    #[rstest]
    #[case(r#"<img src="w3c.png" alt="" aria-labelledby="label"><span hidden id="label">W3C logo</span>"#)]
    #[case(r#"<svg role="none" aria-label="Yellow circle"><circle r="40"></circle></svg>"#)]
    #[case(r#"<nav role="presentation" aria-label="global"><a href="/">ACT</a></nav>"#)]
    #[case(r#"<a href="/" role="none">Home</a>"#)]
    fn exposed_decorative_elements_fail_46ca7f(#[case] html: &str) {
        assert_eq!(count(check_decorative_exposed, html), 1, "{html}");
    }

    #[rstest]
    #[case(r#"<img src="w3c.png" alt="">"#)]
    #[case(r#"<img src="w3c.png" alt="" aria-hidden="true">"#)]
    #[case(r#"<img src="w3c.png" alt="" hidden>"#)]
    #[case(r#"<nav role="presentation"><a href="/" aria-label="ACT">ACT</a></nav>"#)]
    #[case(r#"<img src="w3c.png" role="presentation" alt="W3C logo">"#)]
    #[case(r#"<svg role="none"><circle r="40"></circle></svg>"#)]
    #[case(r#"<img src="w3c.png" aria-label="W3C logo">"#)]
    fn hidden_or_unexposed_decorative_elements_pass_46ca7f(#[case] html: &str) {
        assert_eq!(count(check_decorative_exposed, html), 0, "{html}");
    }

    #[test]
    fn decorative_message_names_the_exposing_attribute() {
        let page = page_from_html(r#"<svg role="none" aria-label="Yellow circle"></svg>"#);
        let findings = check_decorative_exposed(&page, &CheckOptions::default());
        assert_eq!(
            findings[0].message,
            r#"<svg role="none"> is marked decorative but is exposed by aria-label="Yellow circle""#
        );
    }

    #[rstest]
    #[case(r#"<input type="image" name="search" src="search.svg">"#, 1)]
    #[case(r#"<input type="image" src="search.svg" alt="">"#, 1)]
    #[case(r#"<input type="image" src="search.svg" aria-labelledby="none">"#, 1)]
    #[case(r#"<input type="image" src="search.svg" alt="Search">"#, 0)]
    #[case(r#"<input type="image" src="search.svg" aria-label="Search">"#, 0)]
    #[case(r#"<input type="image" src="search.svg" title="Search">"#, 0)]
    #[case(
        r#"<input type="image" src="s.svg" aria-labelledby="i"><div id="i">Search</div>"#,
        0
    )]
    #[case(r#"<input type="image" src="search.svg" aria-hidden="true">"#, 0)]
    #[case(r#"<button><img src="search.svg" alt="Search"></button>"#, 0)]
    fn image_buttons_59796f(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_image_button_name, html), expected, "{html}");
    }

    #[rstest]
    #[case(r#"<svg role="img"><circle r="40"></circle></svg>"#, 1)]
    #[case(r#"<svg role="img"><title></title><circle r="40"></circle></svg>"#, 1)]
    #[case(r#"<svg><circle role="graphics-symbol" r="40"></circle></svg>"#, 1)]
    #[case(r#"<svg role="img"><text x="5">1 circle</text></svg>"#, 1)]
    #[case(r#"<svg role="bogus image"></svg>"#, 1)]
    #[case(r#"<svg role="img"><title>1 circle</title></svg>"#, 0)]
    #[case(
        r#"<svg><circle role="graphics-symbol" aria-label="1 circle"></circle></svg>"#,
        0
    )]
    #[case(r#"<svg role="graphics-document"><title>1 circle</title></svg>"#, 0)]
    #[case(r#"<svg><circle r="40"></circle></svg>"#, 0)]
    #[case(r#"<svg role="img" aria-hidden="true"></svg>"#, 0)]
    #[case(r#"<svg><circle role="graphics-object"></circle></svg>"#, 0)]
    #[case(r#"<svg role="presentation img"></svg>"#, 0)]
    fn svg_images_7d6734(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_svg_image_name, html), expected, "{html}");
    }

    #[rstest]
    #[case(r#"<object data="/moon-speech.mp3"></object>"#, 1)]
    #[case(r#"<object title="" data="/video.mp4"></object>"#, 1)]
    #[case(
        r#"<span id="l"></span><object aria-labelledby="l" data="/logo.png"></object>"#,
        1
    )]
    #[case(
        r#"<object data="/logo.png"><img src="/logo.png" alt="W3C logo"></object>"#,
        1
    )]
    #[case(r#"<object data="/moon.mp3" alt="Moon speech"></object>"#, 1)]
    #[case(r#"<object type="video/mp4" data="/stream"></object>"#, 1)]
    #[case(
        r#"<object aria-label="Moon speech" data="/moon-speech.mp3"></object>"#,
        0
    )]
    #[case(r#"<object title="Rabbit" data="/video.mp4?v=2"></object>"#, 0)]
    #[case(r#"<object role="img" data="/logo.png"></object>"#, 0)]
    #[case(r#"<object data="/logo.png" aria-hidden="true"></object>"#, 0)]
    #[case(
        r#"<object type="image/png" role="presentation" data="/e.png"></object>"#,
        0
    )]
    #[case(r#"<object title="My University" data="/index.html"></object>"#, 0)]
    #[case(r#"<object data="/index.html"></object>"#, 0)]
    #[case(r#"<object type="text/html" data="/logo.png"></object>"#, 0)]
    fn objects_8fc3b6(#[case] html: &str, #[case] expected: usize) {
        assert_eq!(count(check_object_name, html), expected, "{html}");
    }
}
