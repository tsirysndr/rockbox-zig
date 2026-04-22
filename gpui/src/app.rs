use crate::controller::Controller;
use crate::state::AppState;
use crate::ui::{assets::Assets, rockbox::Rockbox};
use gpui::{
    px, size, AppContext, Application, Bounds, TitlebarOptions, WindowBounds, WindowOptions,
};

pub fn run() {
    let assets = Assets {};

    Application::new()
        .with_assets(assets.clone())
        .run(move |cx| {
            let bounds = Bounds::centered(None, size(px(1280.0), px(760.0)), cx);
            assets.load_fonts(cx).expect("failed to load fonts");

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    app_id: Some("rockbox".into()),
                    focus: true,
                    titlebar: Some(TitlebarOptions {
                        title: None,
                        appears_transparent: true,
                        ..Default::default()
                    }),
                    window_min_size: Some(gpui::Size {
                        width: px(800.0),
                        height: px(600.0),
                    }),
                    ..Default::default()
                },
                |_window, cx| {
                    let state = cx.new(|_| AppState::new());
                    let controller = Controller::new(state);
                    cx.set_global(controller);
                    cx.new(Rockbox::new)
                },
            )
            .expect("failed to open window");

            cx.activate(true);
        });
}
