#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub enum HorizontalAlign {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(PartialEq, Eq, Default, Clone, Copy)]
pub enum VerticalAlign {
    #[default]
    Up,
    Center,
    Down,
}

#[derive(Bundle)]
pub struct SvarogUIWindowBundle {
    pub sprite: SpriteBundle,
    pub layer: RenderLayers,
}

#[derive(Bundle)]
pub struct SvarogUITileBundle {
    pub sprite: SpriteSheetBundle,
    pub layer: RenderLayers,
}

impl SvarogUIWindowBundle {
    pub fn new(
        color: Color,
        window_size: Vec2,
        texture: Handle<Image>,
        offset: Vec2,
        size: Vec2,
        align: (HorizontalAlign, VerticalAlign),
    ) -> Self {
        let Vec2 { x, y } = offset;
        let Vec2 { x: w, y: h } = size;
        let (hor, ver) = align;

        let (x, y, w, h) = {
            let half_size = Vec2::new((window_size.x - w) / 2.0, -(window_size.y - h) / 2.0);
            let left_top = -half_size;

            let hor_align_offset = match hor {
                HorizontalAlign::Left => 0.0f32,
                HorizontalAlign::Center => 1.0 * half_size.x,
                HorizontalAlign::Right => 2.0 * half_size.x,
            };

            let ver_align_offset = match ver {
                VerticalAlign::Up => 0.0f32,
                VerticalAlign::Center => 1.0 * half_size.y,
                VerticalAlign::Down => 2.0 * half_size.y,
            };

            let x = hor_align_offset + x + left_top.x;
            let y = ver_align_offset + -y + left_top.y;

            (x, y, w, h)
        };

        SvarogUIWindowBundle {
            sprite: SpriteBundle {
                texture,
                sprite: Sprite {
                    color,
                    ..Default::default()
                },
                transform: Transform::from_translation(Vec3::new(x, y, 0.0))
                    .with_scale(Vec3::new(w, h, 1.0)),
                ..Default::default()
            },
            layer: RenderLayers::layer(2),
        }
    }

    pub fn sub_tile(
        &self,
        texture_atlas: Handle<TextureAtlas>,
        index: Tile,
        offset: Vec2,
        align: (HorizontalAlign, VerticalAlign),
    ) -> SvarogUITileBundle {
        let (hor, ver) = align;

        let local_size = (
            16.0 / self.sprite.transform.scale.x,
            16.0 / self.sprite.transform.scale.y,
        );

        let Vec2 { x, y } = offset;
        let x = local_size.0 * x;
        let y = local_size.1 * y;

        let (x, y) = {
            let half_size = Vec2::new((1.0 - local_size.0) / 2.0, -(1.0 - local_size.1) / 2.0);
            let left_top = -half_size;

            let hor_align_offset = match hor {
                HorizontalAlign::Left => 0.0f32,
                HorizontalAlign::Center => 1.0 * half_size.x,
                HorizontalAlign::Right => 2.0 * half_size.x,
            };

            let ver_align_offset = match ver {
                VerticalAlign::Up => 0.0f32,
                VerticalAlign::Center => 1.0 * half_size.y,
                VerticalAlign::Down => 2.0 * half_size.y,
            };

            let x = hor_align_offset + x + left_top.x;
            let y = ver_align_offset + -y + left_top.y;

            (x, y)
        };

        SvarogUITileBundle {
            sprite: SpriteSheetBundle {
                transform: Transform::from_scale(1.0 / self.sprite.transform.scale.abs())
                    .with_translation(Vec3::new(x, y, 1.0)), //offset.x * 0.5, -offset.y * 0.5, 1.0)),
                sprite: TextureAtlasSprite::new(index.into()),
                texture_atlas,
                ..Default::default()
            },
            layer: RenderLayers::layer(2),
        }
    }
}

pub fn sub_tile_template(
    scale: Vec3,
    texture_atlas: Handle<TextureAtlas>,
    align: (HorizontalAlign, VerticalAlign),
) -> SvarogSubUI {
    fn sub(
        index: Tile,
        offset: Vec2,
        scale: Vec3,
        texture_atlas: Handle<TextureAtlas>,
        align: (HorizontalAlign, VerticalAlign),
    ) -> SvarogUITileBundle {
        let (hor, ver) = align;

        let local_size = (16.0 / scale.x, 16.0 / scale.y);

        let Vec2 { x, y } = offset;
        let x = local_size.0 * x;
        let y = local_size.1 * y;

        let (x, y) = {
            let half_size = Vec2::new((1.0 - local_size.0) / 2.0, -(1.0 - local_size.1) / 2.0);
            let left_top = -half_size;

            let hor_align_offset = match hor {
                HorizontalAlign::Left => 0.0f32,
                HorizontalAlign::Center => 1.0 * half_size.x,
                HorizontalAlign::Right => 2.0 * half_size.x,
            };

            let ver_align_offset = match ver {
                VerticalAlign::Up => 0.0f32,
                VerticalAlign::Center => 1.0 * half_size.y,
                VerticalAlign::Down => 2.0 * half_size.y,
            };

            let x = hor_align_offset + x + left_top.x;
            let y = ver_align_offset + -y + left_top.y;

            (x, y)
        };

        SvarogUITileBundle {
            sprite: SpriteSheetBundle {
                transform: Transform::from_scale(1.0 / scale.abs())
                    .with_translation(Vec3::new(x, y, 1.0)), //offset.x * 0.5, -offset.y * 0.5, 1.0)),
                sprite: TextureAtlasSprite::new(index.into()),
                texture_atlas,
                ..Default::default()
            },
            layer: RenderLayers::layer(2),
        }
    }

    SvarogSubUI {
        func: Box::new(move |index: Tile, offset: Vec2| {
            sub(index, offset, scale, texture_atlas.clone_weak(), align)
        }),
        subs: vec![],
    }
}

pub struct SvarogSubUI {
    pub func: Box<dyn Fn(Tile, Vec2) -> SvarogUITileBundle>,
    subs: Vec<SvarogUITileBundle>,
}

impl SvarogSubUI {
    pub fn make(mut self, tile: Tile, offset: Vec2) -> Self {
        self.subs.push((self.func)(tile, offset));
        self
    }

    pub fn spawn(self, c: &mut ChildBuilder) {
        for sub in self.subs {
            c.spawn(sub);
        }
    }
}

/*
fn debug_ui_window(
    mut commands: Commands,
    mut context: NonSendMut<ImguiContext>,
    window: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,
    grid: Option<Res<Grid>>,
    mut xy: Local<(
        f32,
        f32,
        f32,
        f32,
        HorizontalAlign,
        VerticalAlign,
        f32,
        f32,
        f32,
    )>,
) {
    let Some(grid) = grid else {
        return;
    };

    let ui = context.ui();
    let size = {
        let w = window.single();
        (w.width(), w.height())
    };
    let window = ui.window("Debug UI");

    let mut is_left = xy.4 == HorizontalAlign::Center;
    let mut is_up = xy.5 == VerticalAlign::Center;

    window
        .size([300.0, 300.0], imgui::Condition::FirstUseEver)
        .save_settings(true)
        .build(|| {
            ui.input_float("X", &mut xy.0).build();
            ui.input_float("Y", &mut xy.1).build();
            ui.input_float("W", &mut xy.2).build();
            ui.input_float("H", &mut xy.3).build();

            let _ = ui.checkbox("Horizontal Left", &mut is_left);
            let _ = ui.checkbox("Vertical Up", &mut is_up);

            xy.4 = if is_left {
                HorizontalAlign::Center
            } else {
                HorizontalAlign::Right
            };

            xy.5 = if is_up {
                VerticalAlign::Center
            } else {
                VerticalAlign::Down
            };

            let r: f32 = xy.6;
            let g: f32 = xy.7;
            let b: f32 = xy.8;
            let mut rgb = [r, g, b];
            let _ = ui.color_edit3("Color", &mut rgb);

            xy.6 = rgb[0];
            xy.7 = rgb[1];
            xy.8 = rgb[2];

            if ui.button("CREATE!") {
                let bundle = SvarogUIWindowBundle::new(
                    Color::rgb(xy.6, xy.7, xy.8),
                    Vec2::new(size.0, size.1),
                    asset_server.load("black.png"),
                    Vec2::new(xy.0, xy.1),
                    Vec2::new(xy.2, xy.3),
                    (xy.4, xy.5),
                );

                let sub = sub_tile_template(
                    bundle.sprite.transform.scale,
                    grid.atlas.clone_weak(),
                    (HorizontalAlign::Left, VerticalAlign::Up),
                );

                let sub = sub
                    .make(HP_FULL, Vec2::new(1., 1.0))
                    .make(HP_FULL, Vec2::new(2., 1.0))
                    .make(HP_EMPTY, Vec2::new(3., 1.0));

                commands.spawn(bundle).with_children(|c| {
                    sub.spawn(c);
                });
            }
        });
}
 */
