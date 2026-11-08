use std::{cell::RefCell, rc::Rc, time::Duration};
use sveltui::Runtime;
use sveltui::prelude::*;

use crate::image_component::{picture, placeholder};
use crate::image_types::ImageProps as LocalImageProps;
use crate::network_loader::{Entry, request};
use crate::network_types::*;

/// What the parts of one `Root` have seen so far this frame, in order.
#[derive(Clone)]
struct Sources {
    /// Whether each `Image` part so far has failed.
    failed: Rc<RefCell<Vec<bool>>>,
    /// Whether one of them is loaded and being shown.
    shown: Rc<RefCell<bool>>,
    /// Whether one of them is still loading, ahead of anything shown.
    loading: Rc<RefCell<bool>>,
    width: Option<u16>,
    height: Option<u16>,
    fit: crate::ImageFit,
    protocol: crate::ImageProtocol,
    background: Color,
}

/// Owns the box and the loading status. Put its `Image` parts first and its
/// `Fallback` last: parts are tried in the order they appear, and the
/// fallback shows when none of the images has.
#[component]
pub fn Root(props: RootProps) -> VNode {
    let status = crate::controllable::use_controllable(
        props.status.bound(),
        props.status.value(),
        ImageStatus::default,
        props.on_loading_status_change.clone(),
    );
    let sources = Sources {
        failed: Rc::new(RefCell::new(Vec::new())),
        shown: Rc::new(RefCell::new(false)),
        loading: Rc::new(RefCell::new(false)),
        width: props.width,
        height: props.height,
        fit: props.fit,
        protocol: props.protocol,
        background: props.background,
    };
    let children = provide_context(sources.clone(), || props.children.render(()));

    let shown = *sources.shown.borrow();
    let failed = sources.failed.borrow();
    let next = if shown {
        ImageStatus::Loaded
    } else if failed.is_empty() {
        ImageStatus::Idle
    } else if *sources.loading.borrow() {
        ImageStatus::Loading
    } else {
        ImageStatus::Error
    };
    drop(failed);
    status.set(next);

    let mut view = ViewElement::new().overflow(Overflow::Hidden);
    if let Some(width) = props.width {
        view = view.width(Dimension::Length(f32::from(width)));
    }
    if let Some(height) = props.height {
        view = view.height(Dimension::Length(f32::from(height)));
    }
    view.child(children).into_node()
}

/// One source to try. It starts loading when it first renders; if an
/// earlier source failed and this one has loaded, it is drawn.
#[component]
pub fn Image(props: ImageProps) -> VNode {
    let sources =
        use_context::<Sources>().expect("NetworkImage::Image requires NetworkImage::Root");
    // The decoded picture, converted once so it keeps one identity across
    // frames.
    let ready = use_hook(|| Rc::new(RefCell::new(None::<(String, Rc<image::RgbaImage>)>)));

    let entry = request(
        &props.src,
        Duration::from_millis(props.timeout_ms),
        props.max_bytes,
    );
    let earlier_failed = sources.failed.borrow().iter().all(|failed| *failed);
    let failed = matches!(entry, Entry::Failed(_));
    sources.failed.borrow_mut().push(failed);

    match entry {
        Entry::Ready(img) if earlier_failed && !*sources.shown.borrow() => {
            let picture_source = {
                let mut ready = ready.borrow_mut();
                if ready.as_ref().is_none_or(|(src, _)| *src != props.src) {
                    *ready = Some((props.src.clone(), Rc::new((*img).clone())));
                }
                ready.as_ref().map(|(_, img)| img.clone())
            };
            *sources.shown.borrow_mut() = true;
            let local = LocalImageProps {
                src: None,
                bytes: None,
                alt: props.alt.clone(),
                width: sources.width,
                height: sources.height,
                fit: sources.fit,
                protocol: sources.protocol,
                background: sources.background,
            };
            let alt = props.alt.unwrap_or_default();
            let body = match picture_source {
                Some(img) => picture(&local, img),
                None => placeholder(alt.clone()),
            };
            PressableElement::new()
                .semantics(Semantics::new(Role::Image, alt))
                .child(body)
                .into_node()
        }
        Entry::Loading if earlier_failed => {
            *sources.loading.borrow_mut() = true;
            VNode::Empty
        }
        _ => VNode::Empty,
    }
}

/// What shows while no image has: after `delay_ms` if one is loading, at
/// once if they all failed. It goes away when an image loads.
#[component]
pub fn Fallback(props: FallbackProps) -> VNode {
    let sources =
        use_context::<Sources>().expect("NetworkImage::Fallback requires NetworkImage::Root");
    let started = use_hook(std::time::Instant::now);
    let delay = Duration::from_millis(props.delay_ms);
    use_hook(|| {
        // Nothing else repaints when the delay is up, so ask for it.
        if !delay.is_zero() {
            std::thread::spawn(move || {
                std::thread::sleep(delay);
                Runtime::set_dirty(true);
            });
        }
    });

    if *sources.shown.borrow() {
        return VNode::Empty;
    }
    let waiting = *sources.loading.borrow() && started.elapsed() < delay;
    if waiting {
        return VNode::Empty;
    }
    props.children.render(())
}
