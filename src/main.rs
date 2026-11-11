use macroquad::prelude::*;

// Типы войск
#[derive(Clone, Copy, PartialEq)]
enum UnitType {
    Soldier,    // Солдат (зеленый)
    Tank,       // Танк (темно-зеленый)
    Btr,        // БТР (оливковый)
    Uav,        // БПЛА (серый)
    Radar,      // Радар (голубой)
    Helicopter, // Вертолет (желтый)
    Pvo,        // ПВО (красный)
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

    fn stats(&self) -> (f32, f32) {
        // (Радиус, Скорость) - упрощенно
        match self {
            UnitType::Soldier => (8.0, 80.0),
            UnitType::Tank => (12.0, 40.0),
            UnitType::Btr => (10.0, 60.0),
            UnitType::Uav => (6.0, 150.0),
            UnitType::Radar => (10.0, 0.0),
            UnitType::Helicopter => (10.0, 120.0),
            UnitType::Pvo => (10.0, 0.0),
        }
    }
}

struct Unit {
    pos: Vec2,
    unit_type: UnitType,
    selected: bool,
}

#[macroquad::main("Pixel War")]
async fn main() {
    let mut units: Vec<Unit> = Vec::new();
    let mut selected_type = UnitType::Soldier;
    let mut selected_unit_index: Option<usize> = None;

    loop {
        let dt = get_frame_time();
        let (mx, my) = mouse_position();
        let mouse_pos = vec2(mx, my);

        // --- Ввод ---
        // Переключение типа юнита (клавиши 1-7)
        if is_key_pressed(KeyCode::Key1) { selected_type = UnitType::Soldier; }
        if is_key_pressed(KeyCode::Key2) { selected_type = UnitType::Tank; }
        if is_key_pressed(KeyCode::Key3) { selected_type = UnitType::Btr; }
        if is_key_pressed(KeyCode::Key4) { selected_type = UnitType::Uav; }
        if is_key_pressed(KeyCode::Key5) { selected_type = UnitType::Radar; }
        if is_key_pressed(KeyCode::Key6) { selected_type = UnitType::Helicopter; }
        if is_key_pressed(KeyCode::Key7) { selected_type = UnitType::Pvo; }

        // ЛКМ: Выбрать или создать
        if is_mouse_button_pressed(MouseButton::Left) {
            let mut hit = false;
            for (i, unit) in units.iter().enumerate() {
                if unit.pos.distance(mouse_pos) < unit.unit_type.stats().0 {
                    selected_unit_index = Some(i);
                    hit = true;
                    break;
                }
            }
            // Если кликнули по пустому месту — создаем юнита
            if !hit {
                // Ограничиваем спавн "картой" (условно 200..1000 по X, 200..600 по Y)
                if mouse_pos.x > 200.0 && mouse_pos.x < 1000.0 && mouse_pos.y > 200.0 && mouse_pos.y < 600.0 {
                    units.push(Unit {
                        pos: mouse_pos,
                        unit_type: selected_type,
                        selected: false,
                    });
                    selected_unit_index = Some(units.len() - 1);
                }
            }
        }

        // ПКМ: Движение выбранного юнита
        if is_mouse_button_pressed(MouseButton::Right) {
            if let Some(idx) = selected_unit_index {
                if let Some(unit) = units.get_mut(idx) {
                    let (_, speed) = unit.unit_type.stats();
                    if speed > 0.0 {
                        let dir = (mouse_pos - unit.pos).normalize_or_zero();
                        // Простое перемещение (можно сделать физику, но для примера так)
                        let move_dist = speed * dt;
                        unit.pos += dir * move_dist;
                        // Ограничение картой
                        unit.pos.x = unit.pos.x.clamp(200.0, 1000.0);
                        unit.pos.y = unit.pos.y.clamp(200.0, 600.0);
                    }
                }
            }
        }

        // --- Отрисовка ---
        clear_background(LIGHTGRAY);

        // Рисуем "Карту"
        // Условная Россия (север/восток)
        draw_rectangle(200.0, 200.0, 800.0, 200.0, Color::new(0.9, 0.9, 0.9, 1.0)); 
        // Условная Украина (юг/запад)
        draw_rectangle(200.0, 400.0, 800.0, 200.0, Color::new(0.8, 0.8, 0.7, 1.0));
        draw_line(200.0, 400.0, 1000.0, 400.0, 2.0, BLACK); // Граница

        // Рисуем юнитов
        for (i, unit) in units.iter_mut().enumerate() {
            let color = if Some(i) == selected_unit_index { WHITE } else { unit.unit_type.color() };
            let radius = unit.unit_type.stats().0;
            
            // Тень/обводка для выделения
            draw_circle(unit.pos.x, unit.pos.y, radius + 2.0, BLACK);
            draw_circle(unit.pos.x, unit.pos.y, radius, color);
        }

        // UI: Подсказки
        draw_text("1:Soldier 2:Tank 3:BTR 4:UAV 5:Radar 6:Heli 7:PVO", 20.0, 30.0, 20.0, BLACK);
        draw_text("LMB: Spawn/Select | RMB: Move selected", 20.0, 60.0, 20.0, DARKGRAY);
        
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
