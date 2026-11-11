use macroquad::prelude::*;

const TILE_SIZE: f32 = 40.0;
const MAP_WIDTH: usize = 30;
const MAP_HEIGHT: usize = 20;

#[derive(Clone, Copy, PartialEq)]
enum UnitType {
    Soldier,    // Солдат (зеленый) - Наземный
    Tank,       // Танк (темно-зеленый) - Наземный
    Btr,        // БТР (оливковый) - Наземный
    Uav,        // БПЛА (серый) - Воздушный
    Radar,      // Радар (голубой) - Наземный
    Helicopter, // Вертолет (желтый) - Воздушный
    Pvo,        // ПВО (красный) - Наземный
}

impl UnitType {
    fn color(&self) -> Color {
        match self {
            UnitType::Soldier => GREEN,
            UnitType::Tank => DARKGREEN,
            UnitType::Btr => OLIVE,
            UnitType::Uav => GRAY,
            UnitType::Radar => SKYBLUE,
            UnitType::Helicopter => YELLOW,
            UnitType::Pvo => RED,
        }
    }

    // (HP, Скорость, Радиус атаки, Урон, Атакует воздух, Атакует землю)
    fn stats(&self) -> (f32, f32, f32, f32, bool, bool) {
        match self {
            UnitType::Soldier => (100.0, 80.0, 80.0, 15.0, true, true),
            UnitType::Tank => (300.0, 40.0, 120.0, 50.0, false, true),
            UnitType::Btr => (200.0, 70.0, 100.0, 20.0, true, true),
            UnitType::Uav => (40.0, 150.0, 0.0, 0.0, false, false), // Разведка
            UnitType::Radar => (80.0, 0.0, 200.0, 0.0, false, false), // Радиус обнаружения
            UnitType::Helicopter => (150.0, 120.0, 100.0, 30.0, false, true),
            UnitType::Pvo => (120.0, 0.0, 150.0, 60.0, true, false), // Только воздух
        }
    }

    fn is_air(&self) -> bool {
        matches!(self, UnitType::Uav | UnitType::Helicopter)
    }
}

struct Unit {
    pos: Vec2,
    unit_type: UnitType,
    hp: f32,
    max_hp: f32,
    target_pos: Option<Vec2>,
    attack_cooldown: f32,
}

impl Unit {
    fn new(pos: Vec2, unit_type: UnitType) -> Self {
        let (hp, _, _, _, _, _) = unit_type.stats();
        Self {
            pos,
            unit_type,
            hp,
            max_hp: hp,
            target_pos: None,
            attack_cooldown: 0.0,
        }
    }
}

#[macroquad::main("Pixel War")]
async fn main() {
    // Пытаемся загрузить карту
    let map_texture_result = load_texture("assets/map.png").await;
    let has_map = map_texture_result.is_ok();
    let map_texture = map_texture_result.unwrap_or_else(|_| {
        // Если карты нет, создаем пустую заглушку 1x1, чтобы код компилировался
        Texture2D::from_image(&Image::gen_image_color(1, 1, WHITE))
    });

    let mut units: Vec<Unit> = Vec::new();
    let mut selected_type = UnitType::Soldier;
    let mut selected_unit_index: Option<usize> = None;
    
    // Спавним пару стартовых юнитов для теста
    units.push(Unit::new(vec2(5.0 * TILE_SIZE, 5.0 * TILE_SIZE), UnitType::Tank));
    units.push(Unit::new(vec2(10.0 * TILE_SIZE, 5.0 * TILE_SIZE), UnitType::Pvo));
    units.push(Unit::new(vec2(15.0 * TILE_SIZE, 5.0 * TILE_SIZE), UnitType::Radar));

    loop {
        let dt = get_frame_time();
        let (mx, my) = mouse_position();
        let mouse_pos = vec2(mx, my);

        // Привязка к сетке
        let grid_pos = vec2(
            (mouse_pos.x / TILE_SIZE).floor() * TILE_SIZE + TILE_SIZE / 2.0,
            (mouse_pos.y / TILE_SIZE).floor() * TILE_SIZE + TILE_SIZE / 2.0,
        );

        // --- Ввод ---
        if is_key_pressed(KeyCode::Key1) { selected_type = UnitType::Soldier; }
        if is_key_pressed(KeyCode::Key2) { selected_type = UnitType::Tank; }
        if is_key_pressed(KeyCode::Key3) { selected_type = UnitType::Btr; }
        if is_key_pressed(KeyCode::Key4) { selected_type = UnitType::Uav; }
        if is_key_pressed(KeyCode::Key5) { selected_type = UnitType::Radar; }
        if is_key_pressed(KeyCode::Key6) { selected_type = UnitType::Helicopter; }
        if is_key_pressed(KeyCode::Key7) { selected_type = UnitType::Pvo; }

        // ЛКМ: Выбор или спавн
        if is_mouse_button_pressed(MouseButton::Left) {
            let mut hit = false;
            for (i, unit) in units.iter().enumerate() {
                // Радиус клика чуть больше визуального размера юнита для удобства
                if unit.pos.distance(mouse_pos) < TILE_SIZE / 1.5 {
                    selected_unit_index = Some(i);
                    hit = true;
                    break;
                }
            }
            if !hit {
                if grid_pos.x < MAP_WIDTH as f32 * TILE_SIZE && grid_pos.y < MAP_HEIGHT as f32 * TILE_SIZE {
                    units.push(Unit::new(grid_pos, selected_type));
                    selected_unit_index = Some(units.len() - 1);
                }
            }
        }

        // ПКМ: Приказ на движение
        if is_mouse_button_pressed(MouseButton::Right) {
            if let Some(idx) = selected_unit_index {
                if let Some(unit) = units.get_mut(idx) {
                    let (_, speed, _, _, _, _) = unit.unit_type.stats();
                    if speed > 0.0 {
                        unit.target_pos = Some(grid_pos);
                    }
                }
            }
        }

        // --- Логика (Обновление) ---
        let mut damage_queue: Vec<(usize, f32)> = Vec::new();

        for i in 0..units.len() {
            let (speed, atk_range, damage, can_atk_air, can_atk_ground) = {
                let (_, spd, rng, dmg, air, grd) = units[i].unit_type.stats();
                (spd, rng, dmg, air, grd)
            };

            // Движение
            if let Some(target) = units[i].target_pos {
                let dir = target - units[i].pos;
                if dir.length() > 2.0 {
                    let move_dist = speed * dt;
                    units[i].pos += dir.normalize() * move_dist;
                } else {
                    units[i].target_pos = None;
                }
            }

            // Кулдаун
            if units[i].attack_cooldown > 0.0 {
                units[i].attack_cooldown -= dt;
            }

            // Поиск и атака
            if damage > 0.0 && units[i].attack_cooldown <= 0.0 {
                let mut closest_enemy: Option<usize> = None;
                let mut closest_dist = atk_range;

                for j in 0..units.len() {
                    if i == j { continue; }
                    let enemy_is_air = units[j].unit_type.is_air();
                    
                    if enemy_is_air && !can_atk_air { continue; }
                    if !enemy_is_air && !can_atk_ground { continue; }

                    let dist = units[i].pos.distance(units[j].pos);
                    if dist < closest_dist {
                        closest_dist = dist;
                        closest_enemy = Some(j);
                    }
                }

                if let Some(enemy_idx) = closest_enemy {
                    units[i].attack_cooldown = 1.0; // 1 выстрел в секунду
                    damage_queue.push((enemy_idx, damage)); 
                }
            }
        }

        // Наносим урон (вне цикла, чтобы избежать проблем с заимствованием)
        for (idx, dmg) in damage_queue {
            if idx < units.len() {
                units[idx].hp -= dmg;
            }
        }
        
        // Удаляем мертвых
        units.retain(|u| u.hp > 0.0);
        
        // Сбрасываем выделение, если юнит умер
        if let Some(idx) = selected_unit_index {
            if idx >= units.len() {
                selected_unit_index = None;
            }
        }

        // --- Отрисовка ---
        clear_background(BLACK);

        // 1. Отрисовка карты
        if has_map {
            draw_texture(&map_texture, 0.0, 0.0, WHITE);
        } else {
            // Заглушка: Условная Россия (светло-серая) и Украина (светло-желтая)
            draw_rectangle(0.0, 0.0, MAP_WIDTH as f32 * TILE_SIZE, MAP_HEIGHT as f32 * TILE_SIZE * 0.5, Color::new(0.8, 0.8, 0.8, 1.0));
            draw_rectangle(0.0, MAP_HEIGHT as f32 * TILE_SIZE * 0.5, MAP_WIDTH as f32 * TILE_SIZE, MAP_HEIGHT as f32 * TILE_SIZE * 0.5, Color::new(0.9, 0.9, 0.7, 1.0));
            draw_line(0.0, MAP_HEIGHT as f32 * TILE_SIZE * 0.5, MAP_WIDTH as f32 * TILE_SIZE, MAP_HEIGHT as f32 * TILE_SIZE * 0.5, 2.0, BLACK);
        }
        
        // 2. Сетка
        for x in 0..=MAP_WIDTH {
            draw_line(x as f32 * TILE_SIZE, 0.0, x as f32 * TILE_SIZE, MAP_HEIGHT as f32 * TILE_SIZE, 1.0, Color::new(0.0, 0.0, 0.0, 0.15));
        }
        for y in 0..=MAP_HEIGHT {
            draw_line(0.0, y as f32 * TILE_SIZE, MAP_WIDTH as f32 * TILE_SIZE, y as f32 * TILE_SIZE, 1.0, Color::new(0.0, 0.0, 0.0, 0.15));
        }

        // 3. Юниты
        for (i, unit) in units.iter().enumerate() {
            let is_selected = Some(i) == selected_unit_index;
            let (_, _, atk_range, _, _, _) = unit.unit_type.stats();

            // Радиус действия у выбранного
            if is_selected && atk_range > 0.0 {
                draw_circle_lines(unit.pos.x, unit.pos.y, atk_range, 1.0, Color::new(1.0, 0.0, 0.0, 0.5));
            }

            // Тень
            draw_circle(unit.pos.x, unit.pos.y + 2.0, TILE_SIZE / 2.5, Color::new(0.0, 0.0, 0.0, 0.5));
            
            // Основа
            let color = if is_selected { WHITE } else { unit.unit_type.color() };
            draw_circle(unit.pos.x, unit.pos.y, TILE_SIZE / 2.5, color);
            
            // Индикатор "Воздушный" (треугольник)
            if unit.unit_type.is_air() {
                draw_triangle(
                    vec2(unit.pos.x, unit.pos.y - TILE_SIZE / 2.0),
                    vec2(unit.pos.x - 5.0, unit.pos.y - 5.0),
                    vec2(unit.pos.x + 5.0, unit.pos.y - 5.0),
                    BLACK
                );
            }

            // Полоска здоровья (HP)
            let hp_pct = unit.hp / unit.max_hp;
            draw_rectangle(unit.pos.x - TILE_SIZE/2.0, unit.pos.y - TILE_SIZE/1.5, TILE_SIZE, 4.0, RED);
            draw_rectangle(unit.pos.x - TILE_SIZE/2.0, unit.pos.y - TILE_SIZE/1.5, TILE_SIZE * hp_pct, 4.0, GREEN);
        }

        // UI: Подсказки
        draw_text("1:Soldier 2:Tank 3:BTR 4:UAV 5:Radar 6:Heli 7:PVO", 20.0, 30.0, 20.0, WHITE);
        draw_text("LMB: Spawn/Select | RMB: Move selected", 20.0, 60.0, 20.0, LIGHTGRAY);
        
        let type_name = match selected_type {
            UnitType::Soldier => "Soldier",
            UnitType::Tank => "Tank",
            UnitType::Btr => "BTR",
            UnitType::Uav => "UAV",
            UnitType::Radar => "Radar",
            UnitType::Helicopter => "Helicopter",
            UnitType::Pvo => "PVO",
        };
        draw_text(&format!("Selected type: {}", type_name), 20.0, 90.0, 24.0, BLUE);

        next_frame().await
    }
}
