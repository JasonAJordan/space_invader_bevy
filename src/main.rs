use bevy::{
    //core::FixedTimestep,
    prelude::*,
    render::pass::ClearColor,
    sprite::collide_aabb::{collide, Collision},
};
use std::{thread::*, time::*, fmt::Debug};
use rand::*;


#[derive(Debug)]
struct Ship {
    speed: f32,
    shooting: bool,
    scooldown: Timer,
}

#[derive(Debug)]
struct Bullet {
    speed: f32,
}

struct EnemyBullet {
    speed: f32, 
}

#[derive(Debug)]
struct Minnion{
}

#[derive(Debug)]
struct Fleet{
    speed: f32,
    direction: f32,
}

impl Fleet {
    fn hit_wall(&mut self){
        self.speed *= 1.05;
        self.direction *= -1.0; 
    }

    fn hit_wall_2(&mut self){
        self.direction = -1.0 * self.direction; 
    }

    fn speed_up(&mut self){
        self.speed += 0.5;
    }
}

#[derive(Debug)]
struct Boss{
    life: f32, 
    
}

impl Boss {
    fn hit(&mut self){
        self.life -= 1.0;
    }
}

//Hud will contain points as well as some game logic. 
#[derive(Debug)]
struct Hud {
    points: f32,
    boss_mode: bool,
    win: bool,
    lose: bool,
    lives: i16,
}

impl Hud{
    fn addPoint(&mut self, point: f32){
        self.points += point;
    }

    fn spawned_boss(&mut self){
        self.boss_mode = true; 
    }

    fn win(&mut self){
        self.win = true;
    }

    fn lostLife(&mut self, point: i16){
        self.lives -= point;
    }

    fn lose(&mut self){
        self.lose = true; 
    }
}


// initial game setup, this is heavy based around breakout example
fn setup(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    ) {

    // cameras
    commands.spawn_bundle(OrthographicCameraBundle::new_2d());
    commands.spawn_bundle(UiCameraBundle::default());

    //spawn the ship, fleet_id, and hud data
    commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.3, 0.3, 1.0).into()),
            transform: Transform::from_xyz(0.0, -225.0, 0.0),
            sprite: Sprite::new(Vec2::new(30.0, 30.0)),
            ..Default::default()
            })
        .insert(Ship { speed: 500.0, 
                        shooting: false,
                        scooldown: Timer::from_seconds(0.2, false),
                        })
        //.insert(Collider::Ship)
        .insert(Fleet {
            speed: 2.0,
            direction: 1.0,
        })
        .insert(Hud {
            points: 0.0,
            boss_mode: false,
            win: false,
            lives: 3,
            lose: false,
        });

    // score text
    commands.spawn_bundle(TextBundle {
        text: Text {
            sections: vec![
                TextSection {
                    value: "Score: ".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraSans-Bold.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(0.5, 0.5, 1.0),
                    },
                },
                TextSection {
                    value: "".to_string(),
                    style: TextStyle {
                        font: asset_server.load("fonts/FiraMono-Medium.ttf"),
                        font_size: 40.0,
                        color: Color::rgb(1.0, 0.5, 0.5),
                    },
                },
            ],
            ..Default::default()
        },
        style: Style {
            position_type: PositionType::Absolute,
            position: Rect {
                top: Val::Px(5.0),
                left: Val::Px(5.0),
                ..Default::default()
            },
            ..Default::default()
        },
        ..Default::default()
    });
    
    //spawn the first wave of minnions. 
    let x_offset = -350.;
    let y_offset = 50.;
    for x in 0..10 {
        for y in 0..4 {
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(Color::rgb(0.9, 0.5, 0.3).into()),
                    transform: Transform::from_xyz(x_offset + x as f32 * 70.0,
                                                    y_offset + y  as f32 * 70.0,
                                                    0.0),
                    sprite: Sprite::new(Vec2::new(30.0, 30.0)),
                    ..Default::default()
                })
                .insert(Minnion {});
                //.insert(Collider::Minnion);
        }
    }

    // Add walls
    let wall_material = materials.add(Color::rgb(0.8, 0.8, 0.8).into());
    let wall_thickness = 10.0;
    let bounds = Vec2::new(1200.0, 600.0);

    // left
    commands
    .spawn_bundle(SpriteBundle {
        material: wall_material.clone(),
        transform: Transform::from_xyz(-bounds.x / 2.0, 0.0, 0.0),
        sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y + wall_thickness)),
        ..Default::default()
    });
    
    // right
    commands
        .spawn_bundle(SpriteBundle {
            material: wall_material.clone(),
            transform: Transform::from_xyz(bounds.x / 2.0, 0.0, 0.0),
            sprite: Sprite::new(Vec2::new(wall_thickness, bounds.y + wall_thickness)),
            ..Default::default()
    });
   
    // bottom
    commands
    .spawn_bundle(SpriteBundle {
        material: wall_material.clone(),
        transform: Transform::from_xyz(0.0, -bounds.y / 2.0, 0.0),
        sprite: Sprite::new(Vec2::new(bounds.x + wall_thickness, wall_thickness)),
        ..Default::default()
    });
    
    // top
    commands
        .spawn_bundle(SpriteBundle {
            material: wall_material,
            transform: Transform::from_xyz(0.0, bounds.y / 2.0, 0.0),
            sprite: Sprite::new(Vec2::new(bounds.x + wall_thickness, wall_thickness)),
            ..Default::default()
        });
    

}

fn ship_control_system(
    time: Res<Time>, 
    keyboard_input: Res<Input<KeyCode>>,
    mut query: Query<(&mut Ship, &mut Transform)>,
    mut hud_data: Query<&mut Hud>, 
    )   {
    let mut hud = hud_data.iter_mut().next().unwrap();
    
    if hud.lose == false {
        let mut direction = 0.0;
        let mut fired = false; 
        if keyboard_input.pressed(KeyCode::Left) {
            direction -= 1.0;
        }

        if keyboard_input.pressed(KeyCode::Right) {
            direction += 1.0;
        }

        if keyboard_input.pressed(KeyCode::Space){
            
            //spawn_bullets.system();
            fired = true; 
        }

        for (mut ship, mut transform) in query.iter_mut() {
            let translation = &mut transform.translation;
            // move the ship horizontally
            translation.x += direction * ship.speed * time.delta_seconds();
            // bound the ship within the walls
            translation.x = translation.x.min(440.0).max(-440.0);

            // Updates for ship's attack
            ship.shooting = fired;
        }
    }

}

fn spawn_bullets(
    mut commands: Commands,
    time: Res<Time>, 
    mut materials: ResMut<Assets<ColorMaterial>>,
    mut query: Query<(&mut Ship, &Transform)>,
    ){

    if let Some((mut ship, transform)) = query.iter_mut().next() {

        ship.scooldown.tick(Duration::from_secs_f32(time.delta_seconds()));
        if ship.scooldown.finished() && ship.shooting {

            let x_pos = transform.translation[0]; 
            commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(Color::rgb(0.8, 0.8, 0.2).into()),
                    transform: Transform::from_xyz(x_pos, -200.0 , 0.0),
                    sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                    ..Default::default()
                    })
                .insert(Bullet { speed: 500.0 });
            ship.shooting = false; 
            ship.scooldown.reset();
        }
    }

}

fn bullet_behavior(
    mut commands: Commands,
    //time: Res<Time>,
    mut bullet_posistions: Query<(Entity, &Bullet, &mut Transform)>,
    ) {
        for (b_entity, bullet, mut transform) in bullet_posistions.iter_mut(){
            transform.translation.y += 4.;

            if transform.translation.y >= 285.0 {
                commands.entity(b_entity).despawn();
            }

        }  
}

fn minnion_behavior(
    mut minnion_posistions: Query<(Entity, &mut Minnion, &mut Transform,)>,
    mut fleet_control: Query<&mut Fleet>,
    )   {
        let mut fleet = fleet_control.iter_mut().next().unwrap();
        let mut move_down = false; 
       

        for (_e, _minnion, _transform) in minnion_posistions.iter_mut(){
            let translation = _transform.translation;
            
            if translation.x >= 500.0 && fleet.speed <= 15.0 {
                fleet.hit_wall();
                move_down = true; 
                break;
            } else if translation.x >= 500.0 {
                fleet.hit_wall_2();
                move_down = true; 
                break;
            } else if translation.x <= -500.0 && fleet.speed >= -15.0 {
                fleet.hit_wall();
                move_down = true; 
                break;
            }else if translation.x <= -500.0 {
                fleet.hit_wall_2();
                move_down = true; 
                break;
            }
        }
        
        for (_e, _minnion, mut transform) in minnion_posistions.iter_mut(){
            //println!("{:?}", minnion);
            transform.translation.x +=  fleet.speed * fleet.direction;
            
            if move_down {
                transform.translation.y -= 10.0;
            }
        }
}

fn minnion_shoot(
    mut minnions_q: Query<(&Minnion, &Transform,)>,
    mut commands: Commands,
    //time: Res<Time>, 
    mut materials: ResMut<Assets<ColorMaterial>>,
    )   {
        //x will determine which minnion will fire! 
        let x: f32 = rand::random();
        //y will determine the bullet frequency
        let y: f32 = rand::random();
        let count = minnions_q.iter().count() as f32;
        let rand: f32 = x * count;
        //println!("{:?}", rand.round());

        //let test = minnions_q.iter()[rand.round()];
        let mut count: f32 = 0.0;
        //println!("{:?}", test);

        for (_minnion, transform) in minnions_q.iter_mut(){
            
            if count == rand.round() && y >= 0.98 {
                let x_pos = transform.translation[0]; 
                let y_pos = transform.translation[1]; 
                commands
                .spawn_bundle(SpriteBundle {
                    material: materials.add(Color::rgb(0.2, 0.8, 0.8).into()),
                    transform: Transform::from_xyz(x_pos, y_pos , 0.0),
                    sprite: Sprite::new(Vec2::new(10.0, 10.0)),
                    ..Default::default()
                    })
                .insert(EnemyBullet { speed: 300.0 });
            }
            count += 1.0; 
        }

}

// Pretty much the same code as bullet_behavior
fn minnion_bullet_behavior(
    mut commands: Commands,
    //time: Res<Time>,
    mut bullet_posistions: Query<(Entity, &EnemyBullet, &mut Transform)>,
    ) {
        for (b_entity, bullet, mut transform) in bullet_posistions.iter_mut(){
            transform.translation.y -= 4.;
            if transform.translation.y <= -285.0 {
                commands.entity(b_entity).despawn();
            }
        }  
}


//This rewrites the text in the app.
fn scoreboard_system(
    mut hud_data: Query<&mut Hud>, 
    mut query: Query<&mut Text>,
    ) {
    let mut text = query.single_mut().unwrap();
    let mut hud = hud_data.iter_mut().next().unwrap();
    //println!("Asdf {:?}", hud.points);
    text.sections[0].value = format!("Score: {} Lives: {}", hud.points, hud.lives);
}


fn check_minnion_hit(
    mut commands: Commands,
    mut bullet_posistions: Query<(Entity, &mut Bullet, &Transform)>,
    mut minnion_posistions: Query<(Entity, &mut Minnion, &Transform)>,
    mut fleet_control: Query<&mut Fleet>,
    mut hud_data: Query<&mut Hud>, 
    )   {
        let mut fleet = fleet_control.iter_mut().next().unwrap(); 
        let mut hud = hud_data.iter_mut().next().unwrap();

        for (entity_minnion, _minnion, minnion_transform) in minnion_posistions.iter_mut() {
            for (entity_bullet, _bullet, bullet_transfrom) in bullet_posistions.iter_mut()  {

                if collided( &minnion_transform.translation, &bullet_transfrom.translation, 30.0){
                    commands.entity(entity_minnion).despawn();
                    commands.entity(entity_bullet).despawn();
                    //scoreboard.score += 1;
                    if fleet.speed.abs() <= 7.0{
                        fleet.speed_up();
                    }
                    hud.addPoint(1.0);
                    
                }
            }
        }
}

fn check_player_hit(
    mut commands: Commands,
    mut e_bullet_posistions: Query<(Entity, &mut EnemyBullet, &Transform)>,
    mut ship_posistion: Query<(Entity, &mut Ship, &Transform)>,
    mut fleet_control: Query<&mut Fleet>,
    mut hud_data: Query<&mut Hud>, 
    )   {
        let mut fleet = fleet_control.iter_mut().next().unwrap(); 
        let mut hud = hud_data.iter_mut().next().unwrap();

        for (entity_ship, _ship, ship_transform) in ship_posistion.iter_mut() {
            for (entity_bullet, _bullet, bullet_transfrom) in e_bullet_posistions.iter_mut()  {

                if collided( &ship_transform.translation, &bullet_transfrom.translation, 30.0){
                    commands.entity(entity_bullet).despawn();
                    hud.lostLife(1);
                    
                }
            }
        }
}


//I found this line off another rust game, bevy's collider isn't giving me the results I want. 
fn collided(t1: &Vec3, t2: &Vec3, dist: f32) -> bool {
    f32::abs(t1.x - t2.x) <= dist && f32::abs(t1.y - t2.y) <= dist
}


/// If all enemies are gone, spawn boss. 
fn check_for_minnions(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    minnion: Query<&Minnion>,
    mut hud_data: Query<&mut Hud>, 
    ) {
    let mut hud = hud_data.iter_mut().next().unwrap();

    //This prevents the game logic from spawning the boss non stop. 
    if minnion.iter().count() == 0  && hud.boss_mode == false{
        commands
        .spawn_bundle(SpriteBundle {
            material: materials.add(Color::rgb(0.3, 0.3, 1.0).into()),
            transform: Transform::from_xyz(0.0, 100.0, 0.0),
            sprite: Sprite::new(Vec2::new(150.0, 100.0)),
            ..Default::default()
            })
        .insert(Boss {
            life: 10.0, 
        });

        hud.spawned_boss(); 
    }
}


fn check_boss_hit(
    mut commands: Commands,
    mut bullet_posistions: Query<(Entity, &mut Bullet, &Transform)>,
    mut boss_posistions: Query<(Entity, &mut Boss, &Transform)>,
    mut fleet_control: Query<&mut Fleet>,
    mut hud_data: Query<&mut Hud>, 
    )   {
        let mut fleet = fleet_control.iter_mut().next().unwrap(); 
        let mut hud = hud_data.iter_mut().next().unwrap();

        for (entity_boss, mut boss, boss_transform) in boss_posistions.iter_mut() {
            for (entity_bullet, _bullet, bullet_transfrom) in bullet_posistions.iter_mut()  {

                if collided( &boss_transform.translation, &bullet_transfrom.translation, 50.0){
                    //println!("{:?}", boss.life);

                    if boss.life == 0.0 {
                        commands.entity(entity_boss).despawn();
                        hud.addPoint(10.0);
                        hud.win();
                    } else{
                        boss.hit();
                        commands.entity(entity_bullet).despawn();
                    }
                }
            }
        }
}

fn check_game_win(
    mut hud_data: Query<&mut Hud>,
    mut commands: Commands,
    mut query: Query<&mut Text>,
    )   {
        let mut text = query.single_mut().unwrap();
        let mut hud = hud_data.iter_mut().next().unwrap();
        
        if hud.win == true {
            text.sections[1].value = format!("       You Win!!!");
        }
}

fn check_game_over(
    mut commands: Commands,
    mut hud_data: Query<&mut Hud>,
    mut ship_q: Query<(Entity, &mut Ship, &Transform)>,
    mut query: Query<&mut Text>,
    )   {
        let mut text = query.single_mut().unwrap();
        let mut hud = hud_data.iter_mut().next().unwrap();

        if hud.lives == 0 {
            text.sections[1].value = format!("       You lose :(");
            hud.lose();
            // Bevy doesn't like it when I delete the ship (for now)
            // for (entity_ship, ship, _transform) in ship_q.iter_mut() {
            //     commands.entity(entity_ship).despawn();
            // }
        }

    }



// Implementation of "Space Invaders"
fn main() {
    App::build()
        //game setup
        .add_plugins(DefaultPlugins)
        .insert_resource(WindowDescriptor {
            title: "Space Invaders".to_string(),
            vsync: true,
            ..Default::default()
        })
        .insert_resource(ClearColor(Color::rgb(0.9, 0.9, 0.9)))
        .add_startup_system(setup.system())

        //Main game loop
        .add_system(ship_control_system.system())
        .add_system(spawn_bullets.system())
        .add_system(bullet_behavior.system())
        .add_system(minnion_behavior.system())
        .add_system(check_minnion_hit.system())
        .add_system(minnion_shoot.system())
        .add_system(check_player_hit.system())
        .add_system(minnion_bullet_behavior.system())
        .add_system(scoreboard_system.system())
        .add_system(check_for_minnions.system())
        .add_system(check_boss_hit.system())
        .add_system(check_game_win.system())
        .add_system(check_game_over.system())
        .add_system(bevy::input::system::exit_on_esc_system.system())
        .run();
}



