//! Keeps the custom (undecorated) title bar window truly borderless on Windows.
//!
//! winit gives an undecorated window the `WS_CAPTION | WS_BORDER` styles and hides the
//! non-client area by spreading the client rect over it (`WM_NCCALCSIZE`). On Windows 10
//! the desktop compositor still reserves that frame for the window surface: the content
//! is drawn offset by the caption and border, leaving black bars at the window's top-left,
//! while the pointer keeps the un-offset client coordinates, so mouse clicks and hovers
//! are vertically shifted.
//!
//! [`service`] strips `WS_CAPTION` and `WS_BORDER` and forces Windows to recalculate the frame.
//! winit puts them back whenever it changes the window flags (maximize, restore, etc.),
//! so the returned guard checks and reapplies every frame.

pub use platform::service;

pub type WindowFrameFn = Box<dyn FnMut() + 'static>;

#[cfg(target_os = "windows")]
mod platform {
    use super::WindowFrameFn;
    use raw_window_handle::{HasWindowHandle, RawWindowHandle};
    use winsafe::prelude::Handle;
    use winsafe::{co, AtomStr, HwndPlace, HWND, POINT, SIZE};

    type Rect = (i32, i32, i32, i32);

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Edge {
        Left,
        Top,
        Right,
        Bottom,
    }

    pub fn service(cc: &eframe::CreationContext<'_>) -> Option<WindowFrameFn> {
        let target = match cc.window_handle().ok()?.as_raw() {
            RawWindowHandle::Win32(h) => h.hwnd.get(),
            _ => return None,
        };
        let hwnd = find_window(target)?;
        reapply(&hwnd);
        Some(Box::new(move || reapply(&hwnd)))
    }

    fn find_window(target: isize) -> Option<HWND> {
        let mut found = None;
        let _ = winsafe::EnumWindows(|hwnd| {
            if hwnd.ptr() as isize == target {
                found = Some(hwnd);
                false
            } else {
                true
            }
        });
        found
    }

    fn reapply(hwnd: &HWND) {
        strip(hwnd);
        keep_clear_of_the_taskbar(hwnd);
    }

    fn strip(hwnd: &HWND) {
        let style = hwnd.style();
        let without = borderless(style);
        if without.raw() == style.raw() {
            return;
        }
        hwnd.set_style(without);
        let _ = hwnd.SetWindowPos(
            HwndPlace::None,
            POINT::new(),
            SIZE::new(),
            co::SWP::FRAMECHANGED
                | co::SWP::NOCOPYBITS
                | co::SWP::NOMOVE
                | co::SWP::NOSIZE
                | co::SWP::NOZORDER
                | co::SWP::NOACTIVATE,
        );
    }

    const FRAME: i32 = 32;

    fn keep_clear_of_the_taskbar(hwnd: &HWND) {
        if !hwnd.IsZoomed() {
            return;
        }
        let (Ok(r), Ok(mi)) = (
            hwnd.GetWindowRect(),
            hwnd.MonitorFromWindow(co::MONITOR::DEFAULTTONEAREST)
                .GetMonitorInfo(),
        ) else {
            return;
        };
        let (m, w) = (mi.rcMonitor, mi.rcWork);
        let (rect, monitor, work) = (
            (r.left, r.top, r.right, r.bottom),
            (m.left, m.top, m.right, m.bottom),
            (w.left, w.top, w.right, w.bottom),
        );
        let Some((left, top, right, bottom)) = clamped_rect(rect, monitor, work).or_else(|| {
            covers(rect, monitor)
                .then(|| short_of_autohide_taskbars(rect, monitor, work, &taskbar_edges(monitor)))
                .flatten()
        }) else {
            return;
        };
        let (Some(width), Some(height)) = (right.checked_sub(left), bottom.checked_sub(top)) else {
            return;
        };
        let _ = hwnd.SetWindowPos(
            HwndPlace::None,
            POINT { x: left, y: top },
            SIZE {
                cx: width,
                cy: height,
            },
            co::SWP::FRAMECHANGED | co::SWP::NOCOPYBITS | co::SWP::NOZORDER | co::SWP::NOACTIVATE,
        );
        let _ = hwnd.InvalidateRect(None, false);
    }

    fn clamped_rect(
        rect: (i32, i32, i32, i32),
        monitor: (i32, i32, i32, i32),
        work: (i32, i32, i32, i32),
    ) -> Option<(i32, i32, i32, i32)> {
        let (l, t, r, b) = rect;
        let (ml, mt, mr, mb) = monitor;
        let (wl, wt, wr, wb) = work;
        if ml >= mr || mt >= mb || wl < ml || wt < mt || wr > mr || wb > mb || wl >= wr || wt >= wb
        {
            return None;
        }
        let border = [
            (ml == wl).then(|| ml.checked_sub(l)).flatten(),
            (mt == wt).then(|| mt.checked_sub(t)).flatten(),
            (mr == wr).then(|| r.checked_sub(mr)).flatten(),
            (mb == wb).then(|| b.checked_sub(mb)).flatten(),
        ]
        .into_iter()
        .flatten()
        .filter(|n| (0..=FRAME).contains(n))
        .min()?;
        let parked = |monitor: Option<i32>, work: Option<i32>| {
            monitor == Some(border) || work == Some(border)
        };
        if !parked(ml.checked_sub(l), wl.checked_sub(l))
            || !parked(mt.checked_sub(t), wt.checked_sub(t))
            || !parked(r.checked_sub(mr), r.checked_sub(wr))
            || !parked(b.checked_sub(mb), b.checked_sub(wb))
        {
            return None;
        }
        let corrected = (
            wl.checked_sub(border)?,
            wt.checked_sub(border)?,
            wr.checked_add(border)?,
            wb.checked_add(border)?,
        );
        (corrected != rect).then_some(corrected)
    }

    fn covers(rect: Rect, monitor: Rect) -> bool {
        rect.0 <= monitor.0 && rect.1 <= monitor.1 && rect.2 >= monitor.2 && rect.3 >= monitor.3
    }

    fn taskbar_edges(monitor: Rect) -> Vec<Edge> {
        let mut edges = Vec::new();
        for class in ["Shell_TrayWnd", "Shell_SecondaryTrayWnd"] {
            let mut after: Option<HWND> = None;
            for _ in 0..64 {
                let Ok(Some(tray)) =
                    HWND::NULL.FindWindowEx(after.as_ref(), AtomStr::from_str(class), None)
                else {
                    break;
                };
                if let Ok(r) = tray.GetWindowRect() {
                    edges.extend(taskbar_edge((r.left, r.top, r.right, r.bottom), monitor));
                }
                after = Some(tray);
            }
        }
        edges
    }

    fn taskbar_edge(taskbar: Rect, monitor: Rect) -> Option<Edge> {
        let (tl, tt, tr, tb) = (
            i64::from(taskbar.0),
            i64::from(taskbar.1),
            i64::from(taskbar.2),
            i64::from(taskbar.3),
        );
        let (ml, mt, mr, mb) = (
            i64::from(monitor.0),
            i64::from(monitor.1),
            i64::from(monitor.2),
            i64::from(monitor.3),
        );
        if tl >= tr
            || tt >= tb
            || ml >= mr
            || mt >= mb
            || tr <= ml
            || tl >= mr
            || tb <= mt
            || tt >= mb
        {
            return None;
        }
        let (cx, cy) = (tl + tr, tt + tb);
        [
            (Edge::Left, cx - 2 * ml),
            (Edge::Top, cy - 2 * mt),
            (Edge::Right, 2 * mr - cx),
            (Edge::Bottom, 2 * mb - cy),
        ]
        .into_iter()
        .min_by_key(|&(_, distance)| distance)
        .map(|(edge, _)| edge)
    }

    fn short_of_autohide_taskbars(
        rect: Rect,
        monitor: Rect,
        work: Rect,
        taskbars: &[Edge],
    ) -> Option<Rect> {
        if !covers(rect, monitor) {
            return None;
        }
        let (mut l, mut t, mut r, mut b) = rect;
        for edge in taskbars {
            match edge {
                Edge::Left if work.0 == monitor.0 => l = monitor.0.checked_add(1)?,
                Edge::Top if work.1 == monitor.1 => t = monitor.1.checked_add(1)?,
                Edge::Right if work.2 == monitor.2 => r = monitor.2.checked_sub(1)?,
                Edge::Bottom if work.3 == monitor.3 => b = monitor.3.checked_sub(1)?,
                _ => {}
            }
        }
        let trimmed = (l, t, r, b);
        (l < r && t < b && trimmed != rect).then_some(trimmed)
    }

    fn borderless(style: co::WS) -> co::WS {
        style & !(co::WS::CAPTION | co::WS::BORDER)
    }

    #[cfg(test)]
    mod tests {
        use super::{borderless, clamped_rect, short_of_autohide_taskbars, taskbar_edge, Edge};
        use winsafe::co;

        #[test]
        fn the_borderless_style_drops_the_caption_and_border_only() {
            let decorated = co::WS::CAPTION
                | co::WS::BORDER
                | co::WS::SYSMENU
                | co::WS::THICKFRAME
                | co::WS::MAXIMIZEBOX
                | co::WS::MINIMIZEBOX;
            let bare = borderless(decorated);
            assert!(!bare.has(co::WS::CAPTION), "caption gone");
            assert!(!bare.has(co::WS::BORDER), "border gone");
            assert!(bare.has(co::WS::THICKFRAME), "edge resizing kept");
            assert!(
                bare.has(co::WS::MAXIMIZEBOX) && bare.has(co::WS::SYSMENU),
                "window buttons kept"
            );
            assert_eq!(bare.raw(), borderless(bare).raw());
        }

        #[test]
        fn a_maximize_respects_taskbars_on_every_edge() {
            let monitor = (0, 0, 1920, 1080);
            let rect = (-8, -8, 1928, 1088);
            for (work, expected) in [
                ((0, 0, 1920, 1046), (-8, -8, 1928, 1054)),
                ((0, 48, 1920, 1080), (-8, 40, 1928, 1088)),
                ((0, 2, 1920, 1080), (-8, -6, 1928, 1088)),
                ((0, 0, 1920, 1078), (-8, -8, 1928, 1086)),
                ((48, 0, 1920, 1080), (40, -8, 1928, 1088)),
                ((0, 0, 1872, 1080), (-8, -8, 1880, 1088)),
            ] {
                assert_eq!(clamped_rect(rect, monitor, work), Some(expected));
                assert_eq!(
                    clamped_rect(expected, monitor, work),
                    None,
                    "already corrected"
                );
            }
        }

        #[test]
        fn a_top_taskbar_keeps_its_offset_at_other_origins_and_frame_sizes() {
            let monitor = (-1920, -1080, 0, 0);
            let work = (-1920, -1032, 0, 0);
            for border in [0, 8, 16, 32] {
                let rect = (-1920 - border, -1080 - border, border, border);
                let expected = (-1920 - border, -1032 - border, border, border);
                assert_eq!(clamped_rect(rect, monitor, work), Some(expected));
                assert_eq!(clamped_rect(expected, monitor, work), None);
            }
        }

        #[test]
        fn a_maximize_without_a_reserved_taskbar_is_left_alone() {
            let monitor = (0, 0, 1920, 1080);
            assert_eq!(clamped_rect((-8, -8, 1928, 1088), monitor, monitor), None);
        }

        #[test]
        fn a_maximize_being_dragged_or_snapped_is_left_alone() {
            let monitor = (0, 0, 1920, 1080);
            let work = (0, 48, 1920, 1080);
            assert_eq!(
                clamped_rect((100, 200, 1100, 900), monitor, work),
                None,
                "moved"
            );
            assert_eq!(
                clamped_rect((-500, -8, 1428, 1088), monitor, work),
                None,
                "dragged"
            );
            assert_eq!(
                clamped_rect((0, 48, 960, 1080), monitor, work),
                None,
                "half-screen"
            );
        }

        #[test]
        fn a_maximize_leaves_a_pixel_strip_on_an_auto_hidden_taskbar_edge() {
            let monitor = (0, 0, 1920, 1080);
            let rect = (-8, -8, 1928, 1088);
            for (edge, expected) in [
                (Edge::Bottom, (-8, -8, 1928, 1079)),
                (Edge::Top, (-8, 1, 1928, 1088)),
                (Edge::Left, (1, -8, 1928, 1088)),
                (Edge::Right, (-8, -8, 1919, 1088)),
            ] {
                assert_eq!(
                    short_of_autohide_taskbars(rect, monitor, monitor, &[edge]),
                    Some(expected)
                );
                assert_eq!(
                    short_of_autohide_taskbars(expected, monitor, monitor, &[edge]),
                    None,
                    "already trimmed"
                );
            }
            assert_eq!(
                short_of_autohide_taskbars(rect, monitor, monitor, &[]),
                None,
                "no taskbar"
            );
            assert_eq!(
                short_of_autohide_taskbars(rect, monitor, (0, 0, 1920, 1032), &[Edge::Bottom]),
                None
            );
            assert_eq!(
                short_of_autohide_taskbars((0, 0, 960, 1080), monitor, monitor, &[Edge::Bottom]),
                None,
                "half-screen"
            );
        }

        #[test]
        fn a_taskbar_sits_on_the_monitor_edge_nearest_its_centre() {
            let monitor = (0, 0, 1920, 1080);
            assert_eq!(
                taskbar_edge((0, 1032, 1920, 1080), monitor),
                Some(Edge::Bottom),
                "shown"
            );
            assert_eq!(
                taskbar_edge((0, 1078, 1920, 1126), monitor),
                Some(Edge::Bottom),
                "auto-hidden"
            );
            assert_eq!(taskbar_edge((0, -46, 1920, 2), monitor), Some(Edge::Top));
            assert_eq!(taskbar_edge((-60, 0, 2, 1080), monitor), Some(Edge::Left));
            assert_eq!(
                taskbar_edge((1918, 0, 1980, 1080), monitor),
                Some(Edge::Right)
            );
            assert_eq!(
                taskbar_edge((1920, 1032, 3840, 1080), monitor),
                None,
                "other monitor"
            );
            assert!(
                taskbar_edge((i32::MIN, i32::MIN, i32::MAX, i32::MAX), monitor).is_some(),
                "no overflow"
            );
        }
    }
}

#[cfg(not(target_os = "windows"))]
mod platform {
    use super::WindowFrameFn;

    pub fn service(_cc: &eframe::CreationContext<'_>) -> Option<WindowFrameFn> {
        None
    }
}
