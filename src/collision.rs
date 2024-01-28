use std::sync::Mutex;
use macroquad::prelude::*;

#[derive(Clone, Copy, Debug)]
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

    pub fn overlaps(&self, aabb: AABB) -> bool {
        let a = self;
        let b = &aabb;
        let coll_x = (b.x >= a.x && b.x <= a.x + a.w) || (a.x >= b.x && a.x <= b.x + b.w);
        let coll_y = (b.y >= a.y && b.y <= a.y + a.h) || (a.y >= b.y && a.y <= b.y + b.h);
        return coll_x && coll_y;
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

#[derive(Clone)]
struct Objekt {
    aabb: AABB,
    layer: u32,
    mask: u32,
    user_id: u32,
}

pub const LAYER_MAP: u32 = 1 << 0;
pub const LAYER_PLAYER: u32 = 1 << 1;
pub const LAYER_SWORD: u32 = 1 << 2;

pub struct DinamicenAABBRef(usize);
pub struct StaticenAABBRef(usize);

pub struct Physics {
    dinamicni: FreeList<Objekt>,
    staticni: FreeList<Objekt>,
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

    pub fn dodaj_dinamicen_obj(aabb: AABB, layer: u32, mask: u32, user_id: u32) -> DinamicenAABBRef {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        assert_eq!(layer.count_ones(), 1);

        let i = physics.dinamicni.vstavi(Objekt {
            aabb, layer, mask, user_id
        });
        DinamicenAABBRef(i)
    }

    pub fn dodaj_staticen_obj(aabb: AABB, layer: u32, mask: u32) -> StaticenAABBRef {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        assert_eq!(layer.count_ones(), 1);

        let i = physics.staticni.vstavi(Objekt {
            aabb, layer, mask, user_id: 0
        });
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

        let obj = physics.dinamicni.elements[aabb_ref.0].as_mut().unwrap();
        obj.aabb.x += premik.x;
        obj.aabb.y += premik.y;
    }

    pub fn premakni_obj_na(aabb_ref: &DinamicenAABBRef, pozicija: Vec2) {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        let obj = physics.dinamicni.elements[aabb_ref.0].as_mut().unwrap();
        obj.aabb.x = pozicija.x;
        obj.aabb.y = pozicija.y;
    }

    pub fn pozicija_obj(aabb_ref: &DinamicenAABBRef) -> Vec2 {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        let aabb = physics.dinamicni.elements[aabb_ref.0].as_ref().unwrap().aabb;
        Vec2::new(aabb.x, aabb.y)
    }

    // premik_a + premik_b = 1
    fn resi_trk(obj_a: &mut Objekt, obj_b: &mut Objekt, premik_a: f32, premik_b: f32) {
        if (obj_a.layer & obj_b.mask) == 0 || (obj_b.layer & obj_b.mask) == 0 {
            return;
        }

        let a = &mut obj_a.aabb;
        let b = &mut obj_b.aabb;

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
            if let Some(mut obj_i) = physics.dinamicni.elements[i].clone() {
                for j in (i+1)..physics.dinamicni.elements.len() {
                    if let Some(mut obj_j) = physics.dinamicni.elements[j].clone() {
                        resi_trk(&mut obj_i, &mut obj_j, 0.5, 0.5);
                        physics.dinamicni.elements[j] = Some(obj_j);
                    }
                }
                physics.dinamicni.elements[i] = Some(obj_i);
            }
        }

        for i in 0..physics.staticni.elements.len() {
            if let Some(mut obj_i) = physics.staticni.elements[i].clone() {
                for j in 0..physics.dinamicni.elements.len() {
                    if let Some(mut obj_j) = physics.dinamicni.elements[j].clone() {
                        resi_trk(&mut obj_i, &mut obj_j, 0.0, 1.0);
                        physics.dinamicni.elements[j] = Some(obj_j);
                    }
                }
            }
        }
    }

    /// vrne vse dinamicne aabbje v obmocju z id
    pub fn area_query(area: AABB, mask: u32) -> Vec<(u32, AABB)> {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        let mut result = Vec::new();

        for obj in physics.dinamicni.elements.iter() {
            if let Some(o) = obj {
                if (mask & o.layer) > 0 && o.aabb.overlaps(area) {
                    result.push((o.user_id, o.aabb));
                }
            }
        }

        result
    }

    pub fn narisi_aabbje() {
        let mut physics_mutex_guard = GLOBAL_PHYSICS.lock().unwrap();
        let physics = physics_mutex_guard.as_mut().unwrap();

        for obj in physics.dinamicni.elements.iter() {
            if let Some(o) = obj {
                draw_rectangle_lines(o.aabb.x, o.aabb.y, o.aabb.w, o.aabb.h, 1.0, BLUE);
            }
        }

        for obj in physics.staticni.elements.iter() {
            if let Some(o) = obj {
                draw_rectangle_lines(o.aabb.x, o.aabb.y, o.aabb.w, o.aabb.h, 1.0, RED);
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

