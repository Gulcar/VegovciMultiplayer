use std::sync::Mutex;
use macroquad::prelude::*;

#[derive(Clone, Copy)]
pub struct AABB {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

impl AABB {
    pub fn new(x: f32, y: f32, w: f32, h: f32) -> AABB {
        AABB { x, y, w, h }
    }

    pub fn from_vec(pos: Vec2, size: Vec2) -> AABB {
        AABB { x: pos.x, y: pos.y, w: size.x, h: size.y }
    }
}

struct FreeList<T> {
    pub elements: Vec<Option<T>>,
    pub free: Vec<usize>,
}

impl<T> FreeList<T> {
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
            free: Vec::new(),
        }
    }

    pub fn vstavi(&mut self, el: T) -> usize {
        if self.free.len() > 0 {
            let i = self.free.pop().unwrap();
            self.elements[i] = Some(el);
            return i;
        }
        else {
            self.elements.push(Some(el));
            return self.elements.len() - 1;
        }
    }

    pub fn izbrisi(&mut self, index: usize) {
        self.elements[index] = None;
        self.free.push(index);
    }
}

pub struct DinamicenAABBRef(usize);
pub struct StaticenAABBRef(usize);

pub struct Physics {
    dinamicni: FreeList<AABB>,
    staticni: FreeList<AABB>,
}

static GLOBAL_PHYSICS: Mutex<Option<Physics>> = Mutex::new(None);

pub mod physics {
    use crate::collision::*;

    pub fn init() {
        let mut opt = GLOBAL_PHYSICS.lock().unwrap();
        *opt = Some(Physics {
            dinamicni: FreeList::new(),
            staticni: FreeList::new(),
        });
    }

    pub fn dodaj_dinamicen_obj(aabb: AABB) -> DinamicenAABBRef {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        let i = physics.dinamicni.vstavi(aabb);
        DinamicenAABBRef(i)
    }

    pub fn dodaj_staticen_obj(aabb: AABB) -> StaticenAABBRef {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        let i = physics.staticni.vstavi(aabb);
        StaticenAABBRef(i)
    }

    pub fn izbrisi_dinamicen_obj(aabb_ref: &DinamicenAABBRef) {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        physics.dinamicni.izbrisi(aabb_ref.0);
    }

    pub fn izbrisi_staticen_obj(aabb_ref: &StaticenAABBRef) {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        physics.staticni.izbrisi(aabb_ref.0);
    }

    pub fn premakni_obj(aabb_ref: &DinamicenAABBRef, premik: Vec2) {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        let aabb = physics.dinamicni.elements[aabb_ref.0].as_mut().unwrap();
        aabb.x += premik.x;
        aabb.y += premik.y;
    }

    pub fn pozicija_obj(aabb_ref: &DinamicenAABBRef) -> Vec2 {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        let aabb = physics.dinamicni.elements[aabb_ref.0].unwrap();
        Vec2::new(aabb.x, aabb.y)
    }

    // premik_a + premik_b = 1
    fn resi_trk(a: &mut AABB, b: &mut AABB, premik_a: f32, premik_b: f32) {
        let coll_x = (b.x >= a.x && b.x <= a.x + a.w) || (a.x >= b.x && a.x <= b.x + b.w);
        let coll_y = (b.y >= a.y && b.y <= a.y + a.h) || (a.y >= b.y && a.y <= b.y + b.h);
        if coll_x && coll_y {
            let pen_x = f32::min((b.x + b.w - a.x).abs(), (a.x + a.w - b.x).abs());
            let pen_y = f32::min((b.y + b.h - a.y).abs(), (a.y + a.h - b.y).abs());

            if pen_x < pen_y {
                if a.x > b.x {
                    a.x += pen_x * premik_a;
                    b.x -= pen_x * premik_b;
                }
                else {
                    a.x -= pen_x * premik_a;
                    b.x += pen_x * premik_b;
                }
            }
            else {
                if a.y > b.y {
                    a.y += pen_y * premik_a;
                    b.y -= pen_y * premik_b;
                }
                else {
                    a.y -= pen_y * premik_a;
                    b.y += pen_y * premik_b;
                }
            }
        }
    }

    pub fn resi_trke() {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        for i in 0..physics.dinamicni.elements.len() {
            if let Some(mut aabb_i) = physics.dinamicni.elements[i] {
                for j in (i+1)..physics.dinamicni.elements.len() {
                    if let Some(mut aabb_j) = physics.dinamicni.elements[j] {
                        resi_trk(&mut aabb_i, &mut aabb_j, 0.5, 0.5);
                        physics.dinamicni.elements[j] = Some(aabb_j);
                    }
                }
                physics.dinamicni.elements[i] = Some(aabb_i);
            }
        }

        for i in 0..physics.dinamicni.elements.len() {
            if let Some(mut aabb_i) = physics.dinamicni.elements[i] {
                for j in 0..physics.staticni.elements.len() {
                    if let Some(mut aabb_j) = physics.staticni.elements[j] {
                        resi_trk(&mut aabb_i, &mut aabb_j, 1.0, 0.0);
                    }
                }
                physics.dinamicni.elements[i] = Some(aabb_i);
            }
        }
    }

    pub fn narisi_aabbje() {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        for obj in physics.dinamicni.elements.iter() {
            if let Some(aabb) = obj {
                draw_rectangle_lines(aabb.x, aabb.y, aabb.w, aabb.h, 1.0, BLUE);
            }
        }

        for obj in physics.staticni.elements.iter() {
            if let Some(aabb) = obj {
                draw_rectangle_lines(aabb.x, aabb.y, aabb.w, aabb.h, 1.0, RED);
            }
        }
    }

    pub fn st_dinamicnih_obj() -> usize {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        physics.dinamicni.elements.iter()
            .filter(|x| x.is_some())
            .count()
    }

    pub fn st_staticnih_obj() -> usize {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        physics.staticni.elements.iter()
            .filter(|x| x.is_some())
            .count()
    }
}

impl Drop for StaticenAABBRef {
    fn drop(&mut self) {
        physics::izbrisi_staticen_obj(self);
    }
}

impl Drop for DinamicenAABBRef {
    fn drop(&mut self) {
        physics::izbrisi_dinamicen_obj(self);
    }
}

