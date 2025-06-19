use raylib::prelude::*;
use raylib::consts::KeyboardKey::*;

const PINK: [u8; 4] = [255, 0, 255, 255];
const BLUE: [u8; 4] = [0, 0, 255, 255];
const BLACK: [u8; 4] = [0, 0, 0, 255];
const WINDOW_WIDTH: i32 = 480;
const WINDOW_HEIGHT: i32 = 320;
const MAP_WIDTH: i32 = WINDOW_WIDTH / 32; // 15
const MAP_HEIGHT: i32 = WINDOW_HEIGHT / 32; // 10
const WINDOW_PIXLES: usize = (WINDOW_HEIGHT * WINDOW_WIDTH) as usize;
const FOV: f32 = 90.0;

const MAP: [u8; (MAP_WIDTH*MAP_HEIGHT) as usize] = [
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 1, 1, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
];

fn to_rad(x: f32) -> f32 {
    return x * 3.1415926 / 180.0;
}

fn raycast(player_angle: f32, player_x: f32, player_y: f32) -> Vec<[u8; 4]> {
    let mut buf: Vec<[u8; 4]> = vec![BLUE; WINDOW_PIXLES];

    for x in 0..WINDOW_WIDTH {
        let angle = (player_angle + (x as f32 - WINDOW_WIDTH as f32 / 2.0) * (FOV / WINDOW_WIDTH as f32)) % 360.0;
        let rad = to_rad(angle);

        let ray_dir_x = rad.cos();
        let ray_dir_y = rad.sin();

        let mut map_pos_x = (player_x / 32.0).floor();
        let mut map_pos_y = (player_y / 32.0).floor();

        let step_x = if ray_dir_x >= 0.0 { 1.0 } else { -1.0 };
        let step_y = if ray_dir_y >= 0.0 { 1.0 } else { -1.0 };

        let mut side: bool;

        let mut side_dist_x = if step_x > 0.0 {
            (map_pos_x + 1.0 - player_x / 32.0) / ray_dir_x.abs()
        } else {
            (player_x / 32.0 - map_pos_x) / ray_dir_x.abs()
        };

        let mut side_dist_y = if step_y > 0.0 {
            (map_pos_y + 1.0 - player_y / 32.0) / ray_dir_y.abs()
        } else {
            (player_y / 32.0 - map_pos_y) / ray_dir_y.abs()
        };

        let delta_dist_x = 1.0 / ray_dir_x.abs();
        let delta_dist_y = 1.0 / ray_dir_y.abs();

        loop {
            if side_dist_x < side_dist_y {
                side_dist_x += delta_dist_x;
                map_pos_x += step_x;
                side = false;
            } else {
                side_dist_y += delta_dist_y;
                map_pos_y += step_y;
                side = true;
            }

            let map_x = map_pos_x as i32;
            let map_y = map_pos_y as i32;
            if MAP[(map_y * MAP_WIDTH + map_x) as usize] == 1 {
                break;
            }
        }

        let perp_wall_dist = if side {
            (map_pos_y - player_y / 32.0 + (1.0 - step_y) / 2.0) / ray_dir_y
        } else {
            (map_pos_x - player_x / 32.0 + (1.0 - step_x) / 2.0) / ray_dir_x
        };
        //if perp_wall_dist <= 0.0 {
        //    continue; // with collision detection shouldn't be needed
        //}

        let line_height = (WINDOW_HEIGHT as f32 / perp_wall_dist) as i32;
        let draw_start = (-line_height / 2 + WINDOW_HEIGHT / 2).max(0);
        let draw_end = (line_height / 2 + WINDOW_HEIGHT / 2).min(WINDOW_HEIGHT - 1);

        for y in 0..WINDOW_HEIGHT {
            let idx = (y * WINDOW_WIDTH + x) as usize;
            if y >= draw_start && y <= draw_end {
                buf[idx] = PINK; // wall
            } else if y > draw_end {
                buf[idx] = BLACK; // floor
            } 
        }
    }
    return buf;
}

fn is_wall_at(x: f32, y: f32) -> bool {
    let map_x = (x / 32.0) as i32;
    let map_y = (y / 32.0) as i32;

    if map_x < 0 || map_x >= MAP_WIDTH || map_y < 0 || map_y >= MAP_HEIGHT {
        return true; // outside map = wall
    }

    MAP[(map_y * MAP_WIDTH + map_x) as usize] != 0
}

fn main() {
    let (mut rl, thread) = raylib::init()
        .size(WINDOW_WIDTH*2+10, WINDOW_HEIGHT)
        .title("graphics_3d")
        .build();
    rl.disable_cursor();

    let image_left = Image::gen_image_color(WINDOW_WIDTH, WINDOW_HEIGHT, Color::WHITE);
    let mut texture_left: Texture2D = (&mut rl).load_texture_from_image(&thread, &image_left).unwrap();

    let mut player_x = 5.0*32.0;
    let mut player_y = 5.0*32.0;
    let player_speed = 2.0;
    let mut player_angle = 0.0;

    while !rl.window_should_close() {
        let mouse_delta = rl.get_mouse_delta();
        player_angle += mouse_delta.x * 0.1; // Adjust sensitivity here
        if rl.is_key_down(KEY_Q) {
            break;
        }

        if rl.is_key_down(KEY_W) {
            let new_x = player_x + player_speed * to_rad(player_angle).cos();
            let new_y = player_y + player_speed * to_rad(player_angle).sin();
            if !is_wall_at(new_x, player_y) { player_x = new_x; }
            if !is_wall_at(player_x, new_y) { player_y = new_y; }
        }

        if rl.is_key_down(KEY_S) {
            let new_x = player_x - player_speed * to_rad(player_angle).cos();
            let new_y = player_y - player_speed * to_rad(player_angle).sin();
            if !is_wall_at(new_x, player_y) { player_x = new_x; }
            if !is_wall_at(player_x, new_y) { player_y = new_y; }
        }

        if rl.is_key_down(KEY_A) {
            let new_x = player_x - player_speed * to_rad(player_angle + 90.0).cos();
            let new_y = player_y - player_speed * to_rad(player_angle + 90.0).sin();
            if !is_wall_at(new_x, player_y) { player_x = new_x; }
            if !is_wall_at(player_x, new_y) { player_y = new_y; }
        }

        if rl.is_key_down(KEY_D) {
            let new_x = player_x + player_speed * to_rad(player_angle + 90.0).cos();
            let new_y = player_y + player_speed * to_rad(player_angle + 90.0).sin();
            if !is_wall_at(new_x, player_y) { player_x = new_x; }
            if !is_wall_at(player_x, new_y) { player_y = new_y; }
        }

        let mut d = rl.begin_drawing(&thread);
        d.clear_background(Color::BLACK);

        // FOV triangliel
        let ox = WINDOW_WIDTH as f32 + 10.0;
        let tip = Vector2 { x: ox + player_x, y: player_y };
        let angle_rad = to_rad(player_angle);
        let half_fov = to_rad(FOV / 2.0);
        let length = 1000.0; 
        let base_l = Vector2 {
            x: tip.x + (angle_rad - half_fov).cos() * length,
            y: tip.y + (angle_rad - half_fov).sin() * length,
        };
        let base_r = Vector2 {
            x: tip.x + (angle_rad + half_fov).cos() * length,
            y: tip.y + (angle_rad + half_fov).sin() * length,
        };
        d.draw_triangle(tip, base_r, base_l, Color::GRAY);

        // player
        d.draw_circle(player_x as i32 + WINDOW_WIDTH + 10, player_y as i32, 5.0, Color::RED);

        // walls 
        for x in 0..MAP_WIDTH {
            for y in 0..MAP_HEIGHT {
                if MAP[(y*MAP_WIDTH+x) as usize] == 1 {
                    d.draw_rectangle(WINDOW_WIDTH + 10 + (x*32), y*32, 32, 32, Color::ORCHID);
                } 
            }
        }

        // separator
        d.draw_rectangle(WINDOW_WIDTH, 0, 10, WINDOW_HEIGHT, Color::WHITE);

        // 3D
        let mut pixels = Vec::new();
        for color in &raycast(player_angle, player_x, player_y) {
            pixels.extend(color);
        }
        let _ = texture_left.update_texture(&pixels).inspect_err(
            |e| eprintln!("update_texture failed: {e}"
        ));
        d.draw_texture(&texture_left, 0, 0, Color::WHITE);
    }
}
