// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use crate::*;

use std::ops::{Index, IndexMut};

pub enum Object {
    Rock,
    Monster(Mob),
}
use Object::*;

impl Object {
    pub fn render(&self) -> char {
        match self {
            Rock => '#',
            Monster(_) => 'M',
        }
    }

    pub fn collide(&mut self) -> bool {
        match self {
            Rock => true,
            Monster(ref mut mob) => mob.hit(),
        }
    }
}

#[derive(Default)]
pub struct Loc {
    pub object: Option<Object>,
    pub floor: Option<Object>,
}

impl Loc {
    pub fn top(&self) -> Option<&Object> {
        self.object
            .as_ref()
            .and_then(Some)
            .or_else(|| self.floor.as_ref())
    }

    pub fn render(&self) -> char {
        match self.top() {
            Some(obj) => obj.render(),
            None => '.',
        }
    }
}

pub struct Field(Vec<Loc>);

impl Field {
    pub fn establish(&mut self, posn: usize) {
        if self.0.len() <= posn {
            self.0.resize_with(posn + 1, Loc::default);
        }
        assert!(self.0[posn].object.is_none());
    }

    pub fn insert(&mut self, object: Object, posn: usize) {
        self.establish(posn);
        self.0[posn].object = Some(object);
    }

    pub fn has_object(&self, posn: usize) -> bool {
        posn < self.0.len() && self.0[posn].object.is_some()
    }

    pub fn collide(&mut self, posn: usize) -> bool {
        if self.has_object(posn) {
            let status = self.0[posn].object.as_mut().unwrap().collide();
            if !status {
                self.0[posn].object = None;
            }
            status
        } else {
            false
        }
    }

    pub fn render(&self, left: usize, right: usize) -> Vec<char> {
        let cright = right.min(self.0.len());
        let mut result: Vec<char> = self.0[left..cright]
            .iter()
            .map(|loc| loc.render())
            .collect();
        result.resize(right - left, ' ');
        result
    }
}

impl Index<usize> for Field {
    type Output = Loc;

    fn index(&self, index: usize) -> &Loc {
        &self.0[index]
    }
}

impl IndexMut<usize> for Field {
    fn index_mut(&mut self, index: usize) -> &mut Loc {
        &mut self.0[index]
    }
}

impl Default for Field {
    fn default() -> Self {
        let mut field = Vec::new();
        field.push(Loc {
            object: Some(Rock),
            floor: None,
        });
        field.push(Loc::default());
        Field(field)
    }
}
