#![allow(clippy::type_complexity)]
#![allow(clippy::needless_update)]
#![allow(clippy::too_many_arguments)]

use std::error::Error;

use animation::TweeningPlugin;
use bevy::{
    log::{Level, LogPlugin},
    prelude::{
        default, App, AssetPlugin, ClearColor, Color, FixedTime, ImagePlugin, Msaa, PluginGroup, UVec2,
    },
    window::{Cursor, MonitorSelection, Window, WindowPlugin, WindowPosition, WindowResolution},
    DefaultPlugins
};
use bevy_ecs_tilemap::{prelude::TilemapRenderSettings, TilemapPlugin};
use bevy_hanabi::HanabiPlugin;
use common::state::GameState;
use language::{load_language, Language};
use lighting::LightingPlugin;
use parallax::ParallaxPlugin;
use plugins::{
    assets::AssetsPlugin,
    background::BackgroundPlugin,
    camera::{CameraPlugin, UpdateLightEvent},
    cursor::CursorPlugin,
    fps::FpsPlugin,
    inventory::PlayerInventoryPlugin,
    menu::MenuPlugin,
    player::PlayerPlugin,
    settings::{FullScreen, Resolution, SettingsPlugin, VSync},
    ui::PlayerUiPlugin,
    world::WorldPlugin,
};
use rand::seq::SliceRandom;

pub(crate) mod animation;
pub(crate) mod common;
pub(crate) mod items;
pub(crate) mod language;
pub(crate) mod lighting;
pub(crate) mod parallax;
pub(crate) mod plugins;
pub(crate) mod world;

pub use world::WorldSize;

pub fn create_app() -> Result<App, Box<dyn Error>> {
    let language_content = load_language(Language::English)?;
    let title = language_content.titles.choose(&mut rand::thread_rng()).unwrap();

    let mut app = App::new();

    app.add_plugin(SettingsPlugin);

    let resolution = *app.world.resource::<Resolution>();
    let vsync = *app.world.resource::<VSync>();
    let fullscreen = *app.world.resource::<FullScreen>();

    app
        .insert_resource(Msaa::Off)
        .add_plugins(DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window { 
                    cursor: Cursor {
                        visible: false,
                        ..default()
                    },
                    present_mode: vsync.as_present_mode(),
                    mode: fullscreen.as_window_mode(),
                    resolution: WindowResolution::new(resolution.width, resolution.height),
                    title: title.to_owned(),
                    position: WindowPosition::Centered(MonitorSelection::Current),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                watch_for_changes: true,
                ..default()
            })
            .set(LogPlugin {
                level: Level::ERROR,
                filter: "game=debug".to_string(),
            })
            .set(ImagePlugin::default_nearest())
        )
        .insert_resource(TilemapRenderSettings {
            render_chunk_size: UVec2::new(100, 100),
            y_sort: false,
        })
        .insert_resource(language_content)
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(FixedTime::new_from_secs(1. / 60.))
        .add_event::<UpdateLightEvent>()
        .add_state::<GameState>()
        .add_plugin(TweeningPlugin)
        .add_plugin(TilemapPlugin)
        .add_plugin(AssetsPlugin)
        .add_plugin(HanabiPlugin)
        .add_plugin(CursorPlugin)
        .add_plugin(CameraPlugin)
        // .add_plugin(LightingPlugin)
        .add_plugin(ParallaxPlugin)
        .add_plugin(BackgroundPlugin)
        .add_plugin(PlayerUiPlugin)
        .add_plugin(MenuPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(PlayerInventoryPlugin)
        .add_plugin(FpsPlugin)
        .add_plugin(PlayerPlugin);

    #[cfg(feature = "debug")] {
        use plugins::debug::DebugPlugin;
        app.add_plugin(DebugPlugin);
    }

    Ok(app)
}

#[cfg(feature = "terraria_world")]
pub fn generate_terraria_world_file(world_size: WorldSize, seed: u32, world_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    use crate::world::generator::generate_world;

    let world_data = generate_world(seed, world_size);
    world_data.save_as_terraria_world(world_name)
}

#[cfg(feature = "world_image")]
pub fn generate_world_image(world_size: WorldSize, seed: u32, draw_layers: bool) -> Result<(), Box<dyn std::error::Error>> {
    use image::{RgbImage, ImageBuffer, GenericImageView, Pixel};
    use crate::world::{generator::generate_world, wall::Wall};

    fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
        return out_min + (((value - in_min) / (in_max - in_min)) * (out_max - out_min))
    }

    let world_data = generate_world(seed, world_size);

    let size = world_size.size();

    println!("Saving as image...");
    let sky_image = image::io::Reader::open("assets/sprites/backgrounds/Background_0.png")?.decode()?;
    let sky_image_height = sky_image.height() as usize;
    let mut image: RgbImage = ImageBuffer::new(size.width as u32, size.height as u32);

    // Draw walls
    for ((y, x), wall) in world_data.walls.indexed_iter() {
        if let Some(wall) = wall {
            let color = WALL_COLORS[wall.id() as usize];
            image.put_pixel(x as u32, y as u32, image::Rgb(color));
        } else {
            let sky_image_y = map_range(y as f32, 0., size.height as f32, 0., sky_image_height as f32);

            let color = sky_image.get_pixel(0, sky_image_y as u32);

            image.put_pixel(x as u32, y as u32, color.to_rgb());
        }
    }

    // Draw background
    for y in world_data.layer.underground..world_data.size.height {
        for x in 0..world_data.size.width {
            let color = WALL_COLORS[Wall::Dirt.id() as usize];
            image.put_pixel(x as u32, y as u32, image::Rgb(color));
        }
    }

    // Draw blocks
    for ((y, x), block) in world_data.blocks.indexed_iter() {
        if let Some(block) = block {
            let color = BLOCK_COLORS[block.id() as usize];

            image.put_pixel(x as u32, y as u32, image::Rgb(color));
        }
    }

    // Draw layer borders
    if draw_layers {
        let surface_layer = world_data.layer.surface;
        let underground_layer = world_data.layer.underground;
        let cavern_layer = world_data.layer.cavern;

        for x in 0..world_data.size.width {
            image.put_pixel(x as u32, surface_layer as u32, image::Rgb([255, 0, 0]));
            image.put_pixel(x as u32, underground_layer as u32, image::Rgb([255, 0, 0]));
            image.put_pixel(x as u32, cavern_layer as u32, image::Rgb([255, 0, 0]));
        }
    }

    image.save("world.png")?;

    Ok(())
}

#[cfg(feature = "world_image")]
const BLOCK_COLORS: [[u8; 3]; 470] = [
    [151, 107, 75], // 0
    [128, 128, 128], // 1
    [28, 216, 94], // 2
    [27, 197, 109], // 3
    [253, 221, 3], // 4
    [151, 107, 75], // 5
    [140, 101, 80], // 6
    [150, 67, 22], // 7
    [185, 164, 23], // 8
    [185, 194, 195], // 9
    [119, 105, 79], // 10
    [119, 105, 79], // 11
    [174, 24, 69], // 12
    [133, 213, 247], // 13
    [191, 142, 111], // 14
    [191, 142, 111], // 15
    [140, 130, 116], // 16
    [144, 148, 144], // 17
    [191, 142, 111], // 18
    [191, 142, 111], // 19
    [163, 116, 81], // 20
    [233, 207, 94], // 21
    [98, 95, 167], // 22
    [141, 137, 223], // 23
    [122, 116, 218], // 24
    [109, 90, 128], // 25
    [119, 101, 125], // 26
    [226, 196, 49], // 27
    [151, 79, 80], // 28
    [175, 105, 128], // 29
    [170, 120, 84], // 30
    [141, 120, 168], // 31
    [151, 135, 183], // 32
    [253, 221, 3], // 33
    [235, 166, 135], // 34
    [197, 216, 219], // 35
    [230, 89, 92], // 36
    [104, 86, 84], // 37
    [144, 144, 144], // 38
    [181, 62, 59], // 39
    [146, 81, 68], // 40
    [66, 84, 109], // 41
    [251, 235, 127], // 42
    [84, 100, 63], // 43
    [107, 68, 99], // 44
    [185, 164, 23], // 45
    [185, 194, 195], // 46
    [150, 67, 22], // 47
    [128, 128, 128], // 48
    [43, 143, 255], // 49
    [170, 48, 114], // 50
    [192, 202, 203], // 51
    [23, 177, 76], // 52
    [255, 218, 56], // 53
    [200, 246, 254], // 54
    [191, 142, 111], // 55
    [43, 40, 84], // 56
    [68, 68, 76], // 57
    [142, 66, 66], // 58
    [92, 68, 73], // 59
    [143, 215, 29], // 60
    [135, 196, 26], // 61
    [121, 176, 24], // 62
    [110, 140, 182], // 63
    [196, 96, 114], // 64
    [56, 150, 97], // 65
    [160, 118, 58], // 66
    [140, 58, 166], // 67
    [125, 191, 197], // 68
    [190, 150, 92], // 69
    [93, 127, 255], // 70
    [182, 175, 130], // 71
    [182, 175, 130], // 72
    [27, 197, 109], // 73
    [96, 197, 27], // 74
    [36, 36, 36], // 75
    [142, 66, 66], // 76
    [238, 85, 70], // 77
    [121, 110, 97], // 78
    [191, 142, 111], // 79
    [73, 120, 17], // 80
    [245, 133, 191], // 81
    [255, 120, 0], // 82
    [255, 120, 0], // 83
    [255, 120, 0], // 84
    [192, 192, 192], // 85
    [191, 142, 111], // 86
    [191, 142, 111], // 87
    [191, 142, 111], // 88
    [191, 142, 111], // 89
    [144, 148, 144], // 90
    [13, 88, 130], // 91
    [213, 229, 237], // 92
    [253, 221, 3], // 93
    [191, 142, 111], // 94
    [255, 162, 31], // 95
    [144, 148, 144], // 96
    [144, 148, 144], // 97
    [253, 221, 3], // 98
    [144, 148, 144], // 99
    [253, 221, 3], // 100
    [191, 142, 111], // 101
    [229, 212, 73], // 102
    [141, 98, 77], // 103
    [191, 142, 111], // 104
    [144, 148, 144], // 105
    [191, 142, 111], // 106
    [11, 80, 143], // 107
    [91, 169, 169], // 108
    [78, 193, 227], // 109
    [48, 186, 135], // 110
    [128, 26, 52], // 111
    [103, 98, 122], // 112
    [48, 208, 234], // 113
    [191, 142, 111], // 114
    [33, 171, 207], // 115
    [238, 225, 218], // 116
    [181, 172, 190], // 117
    [238, 225, 218], // 118
    [107, 92, 108], // 119
    [92, 68, 73], // 120
    [11, 80, 143], // 121
    [91, 169, 169], // 122
    [106, 107, 118], // 123
    [73, 51, 36], // 124
    [141, 175, 255], // 125
    [159, 209, 229], // 126
    [128, 204, 230], // 127
    [191, 142, 111], // 128
    [255, 117, 224], // 129
    [160, 160, 160], // 130
    [52, 52, 52], // 131
    [144, 148, 144], // 132
    [231, 53, 56], // 133
    [166, 187, 153], // 134
    [253, 114, 114], // 135
    [213, 203, 204], // 136
    [144, 148, 144], // 137
    [96, 96, 96], // 138
    [191, 142, 111], // 139
    [98, 95, 167], // 140
    [192, 59, 59], // 141
    [144, 148, 144], // 142
    [144, 148, 144], // 143
    [144, 148, 144], // 144
    [192, 30, 30], // 145
    [43, 192, 30], // 146
    [211, 236, 241], // 147
    [181, 211, 210], // 148
    [220, 50, 50], // 149
    [128, 26, 52], // 150
    [190, 171, 94], // 151
    [128, 133, 184], // 152
    [239, 141, 126], // 153
    [190, 171, 94], // 154
    [131, 162, 161], // 155
    [170, 171, 157], // 156
    [104, 100, 126], // 157
    [145, 81, 85], // 158
    [148, 133, 98], // 159
    [0, 0, 200], // 160
    [144, 195, 232], // 161
    [184, 219, 240], // 162
    [174, 145, 214], // 163
    [218, 182, 204], // 164
    [100, 100, 100], // 165
    [129, 125, 93], // 166
    [62, 82, 114], // 167
    [132, 157, 127], // 168
    [152, 171, 198], // 169
    [228, 219, 162], // 170
    [33, 135, 85], // 171
    [181, 194, 217], // 172
    [253, 221, 3], // 173
    [253, 221, 3], // 174
    [129, 125, 93], // 175
    [132, 157, 127], // 176
    [152, 171, 198], // 177
    [255, 0, 255], // 178
    [49, 134, 114], // 179
    [126, 134, 49], // 180
    [134, 59, 49], // 181
    [43, 86, 140], // 182
    [121, 49, 134], // 183
    [100, 100, 100], // 184
    [149, 149, 115], // 185
    [255, 0, 255], // 186
    [255, 0, 255], // 187
    [73, 120, 17], // 188
    [223, 255, 255], // 189
    [182, 175, 130], // 190
    [151, 107, 75], // 191
    [26, 196, 84], // 192
    [56, 121, 255], // 193
    [157, 157, 107], // 194
    [134, 22, 34], // 195
    [147, 144, 178], // 196
    [97, 200, 225], // 197
    [62, 61, 52], // 198
    [208, 80, 80], // 199
    [216, 152, 144], // 200
    [203, 61, 64], // 201
    [213, 178, 28], // 202
    [128, 44, 45], // 203
    [125, 55, 65], // 204
    [186, 50, 52], // 205
    [124, 175, 201], // 206
    [144, 148, 144], // 207
    [88, 105, 118], // 208
    [144, 148, 144], // 209
    [192, 59, 59], // 210
    [191, 233, 115], // 211
    [144, 148, 144], // 212
    [137, 120, 67], // 213
    [103, 103, 103], // 214
    [254, 121, 2], // 215
    [191, 142, 111], // 216
    [144, 148, 144], // 217
    [144, 148, 144], // 218
    [144, 148, 144], // 219
    [144, 148, 144], // 220
    [239, 90, 50], // 221
    [231, 96, 228], // 222
    [57, 85, 101], // 223
    [107, 132, 139], // 224
    [227, 125, 22], // 225
    [141, 56, 0], // 226
    [255, 255, 255], // 227
    [144, 148, 144], // 228
    [255, 156, 12], // 229
    [131, 79, 13], // 230
    [224, 194, 101], // 231
    [145, 81, 85], // 232
    [255, 0, 255], // 233
    [53, 44, 41], // 234
    [214, 184, 46], // 235
    [149, 232, 87], // 236
    [255, 241, 51], // 237
    [225, 128, 206], // 238
    [224, 194, 101], // 239
    [99, 50, 30], // 240
    [77, 74, 72], // 241
    [99, 50, 30], // 242
    [140, 179, 254], // 243
    [200, 245, 253], // 244
    [99, 50, 30], // 245
    [99, 50, 30], // 246
    [140, 150, 150], // 247
    [219, 71, 38], // 248
    [249, 52, 243], // 249
    [76, 74, 83], // 250
    [235, 150, 23], // 251
    [153, 131, 44], // 252
    [57, 48, 97], // 253
    [248, 158, 92], // 254
    [107, 49, 154], // 255
    [154, 148, 49], // 256
    [49, 49, 154], // 257
    [49, 154, 68], // 258
    [154, 49, 77], // 259
    [85, 89, 118], // 260
    [154, 83, 49], // 261
    [221, 79, 255], // 262
    [250, 255, 79], // 263
    [79, 102, 255], // 264
    [79, 255, 89], // 265
    [255, 79, 79], // 266
    [240, 240, 247], // 267
    [255, 145, 79], // 268
    [191, 142, 111], // 269
    [122, 217, 232], // 270
    [122, 217, 232], // 271
    [121, 119, 101], // 272
    [128, 128, 128], // 273
    [190, 171, 94], // 274
    [122, 217, 232], // 275
    [122, 217, 232], // 276
    [122, 217, 232], // 277
    [122, 217, 232], // 278
    [122, 217, 232], // 279
    [122, 217, 232], // 280
    [122, 217, 232], // 281
    [122, 217, 232], // 282
    [128, 128, 128], // 283
    [150, 67, 22], // 284
    [122, 217, 232], // 285
    [122, 217, 232], // 286
    [79, 128, 17], // 287
    [122, 217, 232], // 288
    [122, 217, 232], // 289
    [122, 217, 232], // 290
    [122, 217, 232], // 291
    [122, 217, 232], // 292
    [122, 217, 232], // 293
    [122, 217, 232], // 294
    [122, 217, 232], // 295
    [122, 217, 232], // 296
    [122, 217, 232], // 297
    [122, 217, 232], // 298
    [122, 217, 232], // 299
    [144, 148, 144], // 300
    [144, 148, 144], // 301
    [144, 148, 144], // 302
    [144, 148, 144], // 303
    [144, 148, 144], // 304
    [144, 148, 144], // 305
    [144, 148, 144], // 306
    [144, 148, 144], // 307
    [144, 148, 144], // 308
    [122, 217, 232], // 309
    [122, 217, 232], // 310
    [117, 61, 25], // 311
    [204, 93, 73], // 312
    [87, 150, 154], // 313
    [181, 164, 125], // 314
    [235, 114, 80], // 315
    [122, 217, 232], // 316
    [122, 217, 232], // 317
    [122, 217, 232], // 318
    [96, 68, 48], // 319
    [203, 185, 151], // 320
    [96, 77, 64], // 321
    [198, 170, 104], // 322
    [182, 141, 86], // 323
    [228, 213, 173], // 324
    [129, 125, 93], // 325
    [9, 61, 191], // 326
    [253, 32, 3], // 327
    [200, 246, 254], // 328
    [15, 15, 15], // 329
    [226, 118, 76], // 330
    [161, 172, 173], // 331
    [204, 181, 72], // 332
    [190, 190, 178], // 333
    [191, 142, 111], // 334
    [217, 174, 137], // 335
    [253, 62, 3], // 336
    [144, 148, 144], // 337
    [85, 255, 160], // 338
    [122, 217, 232], // 339
    [96, 248, 2], // 340
    [105, 74, 202], // 341
    [29, 240, 255], // 342
    [254, 202, 80], // 343
    [131, 252, 245], // 344
    [255, 156, 12], // 345
    [149, 212, 89], // 346
    [236, 74, 79], // 347
    [44, 26, 233], // 348
    [144, 148, 144], // 349
    [55, 97, 155], // 350
    [31, 31, 31], // 351
    [238, 97, 94], // 352
    [28, 216, 94], // 353
    [141, 107, 89], // 354
    [141, 107, 89], // 355
    [233, 203, 24], // 356
    [168, 178, 204], // 357
    [122, 217, 232], // 358
    [122, 217, 232], // 359
    [122, 217, 232], // 360
    [122, 217, 232], // 361
    [122, 217, 232], // 362
    [122, 217, 232], // 363
    [122, 217, 232], // 364
    [146, 136, 205], // 365
    [223, 232, 233], // 366
    [168, 178, 204], // 367
    [50, 46, 104], // 368
    [50, 46, 104], // 369
    [127, 116, 194], // 370
    [249, 101, 189], // 371
    [252, 128, 201], // 372
    [9, 61, 191], // 373
    [253, 32, 3], // 374
    [255, 156, 12], // 375
    [160, 120, 92], // 376
    [191, 142, 111], // 377
    [160, 120, 100], // 378
    [251, 209, 240], // 379
    [191, 142, 111], // 380
    [254, 121, 2], // 381
    [28, 216, 94], // 382
    [221, 136, 144], // 383
    [131, 206, 12], // 384
    [87, 21, 144], // 385
    [127, 92, 69], // 386
    [127, 92, 69], // 387
    [127, 92, 69], // 388
    [127, 92, 69], // 389
    [253, 32, 3], // 390
    [122, 217, 232], // 391
    [122, 217, 232], // 392
    [122, 217, 232], // 393
    [122, 217, 232], // 394
    [191, 142, 111], // 395
    [198, 124, 78], // 396
    [212, 192, 100], // 397
    [100, 82, 126], // 398
    [77, 76, 66], // 399
    [96, 68, 117], // 400
    [68, 60, 51], // 401
    [174, 168, 186], // 402
    [205, 152, 186], // 403
    [140, 84, 60], // 404
    [140, 140, 140], // 405
    [120, 120, 120], // 406
    [255, 227, 132], // 407
    [85, 83, 82], // 408
    [85, 83, 82], // 409
    [75, 139, 166], // 410
    [227, 46, 46], // 411
    [75, 139, 166], // 412
    [122, 217, 232], // 413
    [122, 217, 232], // 414
    [249, 75, 7], // 415
    [0, 160, 170], // 416
    [160, 87, 234], // 417
    [22, 173, 254], // 418
    [117, 125, 151], // 419
    [255, 255, 255], // 420
    [73, 70, 70], // 421
    [73, 70, 70], // 422
    [255, 255, 255], // 423
    [146, 155, 187], // 424
    [174, 195, 215], // 425
    [77, 11, 35], // 426
    [119, 22, 52], // 427
    [255, 255, 255], // 428
    [63, 63, 63], // 429
    [23, 119, 79], // 430
    [23, 54, 119], // 431
    [119, 68, 23], // 432
    [74, 23, 119], // 433
    [78, 82, 109], // 434
    [39, 168, 96], // 435
    [39, 94, 168], // 436
    [168, 121, 39], // 437
    [111, 39, 168], // 438
    [150, 148, 174], // 439
    [255, 255, 255], // 440
    [255, 255, 255], // 441
    [3, 144, 201], // 442
    [123, 123, 123], // 443
    [191, 176, 124], // 444
    [55, 55, 73], // 445
    [255, 66, 152], // 446
    [179, 132, 255], // 447
    [0, 206, 180], // 448
    [91, 186, 240], // 449
    [92, 240, 91], // 450
    [240, 91, 147], // 451
    [255, 150, 181], // 452
    [255, 255, 255], // 453
    [174, 16, 176], // 454
    [48, 255, 110], // 455
    [179, 132, 255], // 456
    [255, 255, 255], // 457
    [211, 198, 111], // 458
    [190, 223, 232], // 459
    [141, 163, 181], // 460
    [255, 222, 100], // 461
    [231, 178, 28], // 462
    [155, 214, 240], // 463
    [233, 183, 128], // 464
    [51, 84, 195], // 465
    [205, 153, 73], // 466
    [233, 207, 94], // 467
    [255, 255, 255], // 468
    [191, 142, 111] // 469
];

#[cfg(feature = "world_image")]
const WALL_COLORS: [[u8; 3]; 231] = [
    [0, 0, 0], // 0
    [52, 52, 52], // 1
    [88, 61, 46], // 2
    [61, 58, 78], // 3
    [73, 51, 36], // 4
    [52, 52, 52], // 5
    [91, 30, 30], // 6
    [27, 31, 42], // 7
    [31, 39, 26], // 8
    [41, 28, 36], // 9
    [74, 62, 12], // 10
    [46, 56, 59], // 11
    [75, 32, 11], // 12
    [67, 37, 37], // 13
    [15, 15, 15], // 14
    [52, 43, 45], // 15
    [88, 61, 46], // 16
    [27, 31, 42], // 17
    [31, 39, 26], // 18
    [41, 28, 36], // 19
    [15, 15, 15], // 20
    [0, 0, 0], // 21
    [113, 99, 99], // 22
    [38, 38, 43], // 23
    [53, 39, 41], // 24
    [11, 35, 62], // 25
    [21, 63, 70], // 26
    [88, 61, 46], // 27
    [81, 84, 101], // 28
    [88, 23, 23], // 29
    [28, 88, 23], // 30
    [78, 87, 99], // 31
    [86, 17, 40], // 32
    [49, 47, 83], // 33
    [69, 67, 41], // 34
    [51, 51, 70], // 35
    [87, 59, 55], // 36
    [69, 67, 41], // 37
    [49, 57, 49], // 38
    [78, 79, 73], // 39
    [85, 102, 103], // 40
    [52, 50, 62], // 41
    [71, 42, 44], // 42
    [73, 66, 50], // 43
    [52, 52, 52], // 44
    [60, 59, 51], // 45
    [48, 57, 47], // 46
    [71, 77, 85], // 47
    [52, 52, 52], // 48
    [52, 52, 52], // 49
    [52, 52, 52], // 50
    [52, 52, 52], // 51
    [52, 52, 52], // 52
    [52, 52, 52], // 53
    [40, 56, 50], // 54
    [49, 48, 36], // 55
    [43, 33, 32], // 56
    [31, 40, 49], // 57
    [48, 35, 52], // 58
    [88, 61, 46], // 59
    [1, 52, 20], // 60
    [55, 39, 26], // 61
    [39, 33, 26], // 62
    [30, 80, 48], // 63
    [53, 80, 30], // 64
    [30, 80, 48], // 65
    [30, 80, 48], // 66
    [53, 80, 30], // 67
    [30, 80, 48], // 68
    [43, 42, 68], // 69
    [30, 70, 80], // 70
    [78, 105, 135], // 71
    [52, 84, 12], // 72
    [190, 204, 223], // 73
    [64, 62, 80], // 74
    [65, 65, 35], // 75
    [20, 46, 104], // 76
    [61, 13, 16], // 77
    [63, 39, 26], // 78
    [51, 47, 96], // 79
    [64, 62, 80], // 80
    [101, 51, 51], // 81
    [77, 64, 34], // 82
    [62, 38, 41], // 83
    [48, 78, 93], // 84
    [54, 63, 69], // 85
    [138, 73, 38], // 86
    [50, 15, 8], // 87
    [0, 0, 0], // 88
    [0, 0, 0], // 89
    [0, 0, 0], // 90
    [0, 0, 0], // 91
    [0, 0, 0], // 92
    [0, 0, 0], // 93
    [32, 40, 45], // 94
    [44, 41, 50], // 95
    [72, 50, 77], // 96
    [78, 50, 69], // 97
    [36, 45, 44], // 98
    [38, 49, 50], // 99
    [32, 40, 45], // 100
    [44, 41, 50], // 101
    [72, 50, 77], // 102
    [78, 50, 69], // 103
    [36, 45, 44], // 104
    [38, 49, 50], // 105
    [0, 0, 0], // 106
    [0, 0, 0], // 107
    [138, 73, 38], // 108
    [94, 25, 17], // 109
    [125, 36, 122], // 110
    [51, 35, 27], // 111
    [50, 15, 8], // 112
    [135, 58, 0], // 113
    [65, 52, 15], // 114
    [39, 42, 51], // 115
    [89, 26, 27], // 116
    [126, 123, 115], // 117
    [8, 50, 19], // 118
    [95, 21, 24], // 119
    [17, 31, 65], // 120
    [192, 173, 143], // 121
    [114, 114, 131], // 122
    [136, 119, 7], // 123
    [8, 72, 3], // 124
    [117, 132, 82], // 125
    [100, 102, 114], // 126
    [30, 118, 226], // 127
    [93, 6, 102], // 128
    [64, 40, 169], // 129
    [39, 34, 180], // 130
    [87, 94, 125], // 131
    [6, 6, 6], // 132
    [69, 72, 186], // 133
    [130, 62, 16], // 134
    [22, 123, 163], // 135
    [40, 86, 151], // 136
    [183, 75, 15], // 137
    [83, 80, 100], // 138
    [115, 65, 68], // 139
    [119, 108, 81], // 140
    [59, 67, 71], // 141
    [17, 172, 143], // 142
    [90, 112, 105], // 143
    [62, 28, 87], // 144
    [0, 0, 0], // 145
    [120, 59, 19], // 146
    [59, 59, 59], // 147
    [229, 218, 161], // 148
    [73, 59, 50], // 149
    [0, 0, 0], // 150
    [102, 75, 34], // 151
    [0, 0, 0], // 152
    [255, 145, 79], // 153
    [221, 79, 255], // 154
    [240, 240, 247], // 155
    [79, 255, 89], // 156
    [154, 83, 49], // 157
    [107, 49, 154], // 158
    [85, 89, 118], // 159
    [49, 154, 68], // 160
    [154, 49, 77], // 161
    [49, 49, 154], // 162
    [154, 148, 49], // 163
    [255, 79, 79], // 164
    [79, 102, 255], // 165
    [250, 255, 79], // 166
    [70, 68, 51], // 167
    [0, 0, 0], // 168
    [5, 5, 5], // 169
    [59, 39, 22], // 170
    [59, 39, 22], // 171
    [163, 96, 0], // 172
    [94, 163, 46], // 173
    [117, 32, 59], // 174
    [20, 11, 203], // 175
    [74, 69, 88], // 176
    [60, 30, 30], // 177
    [111, 117, 135], // 178
    [111, 117, 135], // 179
    [25, 23, 54], // 180
    [25, 23, 54], // 181
    [74, 71, 129], // 182
    [111, 117, 135], // 183
    [25, 23, 54], // 184
    [52, 52, 52], // 185
    [38, 9, 66], // 186
    [149, 80, 51], // 187
    [82, 63, 80], // 188
    [65, 61, 77], // 189
    [64, 65, 92], // 190
    [76, 53, 84], // 191
    [144, 67, 52], // 192
    [149, 48, 48], // 193
    [111, 32, 36], // 194
    [147, 48, 55], // 195
    [97, 67, 51], // 196
    [112, 80, 62], // 197
    [88, 61, 46], // 198
    [127, 94, 76], // 199
    [143, 50, 123], // 200
    [136, 120, 131], // 201
    [219, 92, 143], // 202
    [113, 64, 150], // 203
    [74, 67, 60], // 204
    [60, 78, 59], // 205
    [0, 54, 21], // 206
    [74, 97, 72], // 207
    [40, 37, 35], // 208
    [77, 63, 66], // 209
    [111, 6, 6], // 210
    [88, 67, 59], // 211
    [88, 87, 80], // 212
    [71, 71, 67], // 213
    [76, 52, 60], // 214
    [89, 48, 59], // 215
    [158, 100, 64], // 216
    [62, 45, 75], // 217
    [57, 14, 12], // 218
    [96, 72, 133], // 219
    [67, 55, 80], // 220
    [64, 37, 29], // 221
    [70, 51, 91], // 222
    [51, 18, 4], // 223
    [57, 55, 52], // 224
    [68, 68, 68], // 225
    [148, 138, 74], // 226
    [95, 137, 191], // 227
    [160, 2, 75], // 228
    [100, 55, 164], // 229
    [0, 117, 101] // 230
];