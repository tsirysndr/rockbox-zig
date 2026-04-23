use crate::ui::global_keybinds::{Hide, HideOthers, Next, PlayPause, Prev, Quit, Repeat, Shuffle};
use crate::ui::global_keybinds::{Library, Player, Queue};
use crate::ui::startup_gate::StartupGate;
use crate::ui::{assets::Assets, theme::Theme};
use gpui::{
    px, size, AppContext, Application, Bounds, Menu, MenuItem, SystemMenuType, TitlebarOptions,
    WindowBounds, WindowOptions,
};

pub fn run() {
    let assets = Assets {};

    let http_rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("tokio runtime for http");
    let http_client = crate::http_client::ReqwestHttpClient::new(http_rt.handle().clone());
    std::mem::forget(http_rt);

    Application::new()
        .with_http_client(http_client)
        .with_assets(assets.clone())
        .run(move |cx| {
            let bounds = Bounds::centered(None, size(px(1280.0), px(760.0)), cx);
            assets.load_fonts(cx).expect("failed to load fonts");
            // Theme is set as a global inside StartupGate / Rockbox::new.
            // Pre-set it here so the error screen can read it before Rockbox initialises.
            cx.set_global(Theme::default());

            cx.open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(bounds)),
                    app_id: Some("Rockbox".into()),
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
                |_window, cx| cx.new(StartupGate::new),
            )
            .expect("failed to open window");

            cx.set_menus(vec![
                Menu {
                    name: "Rockbox".into(),
                    items: vec![
                        MenuItem::os_submenu("Services", SystemMenuType::Services),
                        MenuItem::separator(),
                        MenuItem::action("Hide Rockbox", Hide),
                        MenuItem::action("Hide Others", HideOthers),
                        MenuItem::separator(),
                        MenuItem::action("Quit Rockbox", Quit),
                    ],
                },
                Menu {
                    name: "Playback".into(),
                    items: vec![
                        MenuItem::action("Play / Pause", PlayPause),
                        MenuItem::action("Next Track", Next),
                        MenuItem::action("Previous Track", Prev),
                        MenuItem::separator(),
                        MenuItem::action("Shuffle", Shuffle),
                        MenuItem::action("Repeat", Repeat),
                    ],
                },
                Menu {
                    name: "View".into(),
                    items: vec![
                        MenuItem::action("Library", Library),
                        MenuItem::action("Player", Player),
                        MenuItem::action("Queue", Queue),
                    ],
                },
            ]);

            cx.activate(true);
        });
}
