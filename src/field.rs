// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use crate::*;

use std::ops::{Index, IndexMut};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Object {
    Rock,
    Monster(u64),
    Player(u64),
    Door,
}
use Object::*;

impl Object {
    pub fn render(&self) -> char {
        match self {
            Rock => '#',
            Monster(_) => 'M',
            Player(_) => '@',
            Door => '+',
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
        // XXX I think the closure is really needed, but
        // Clippy doesn't like it.
        #[allow(clippy::unnecessary_lazy_evaluations)]
        self.object.as_ref().or_else(|| self.floor.as_ref())
    }

    pub fn render(&self) -> char {
        match self.top() {
            Some(obj) => obj.render(),
            None => '.',
        }
    }
}

pub struct Field(Vec<Loc>);

#[allow(clippy::len_without_is_empty)]
impl Field {
    pub fn establish(&mut self, posn: usize) {
        if self.len() <= posn {
            self.0.resize_with(posn + 1, Loc::default);
        }
    }

    pub fn insert(&mut self, object: Object, posn: usize) {
        self.establish(posn);
        self[posn].object = Some(object);
    }

    pub fn insert_floor(&mut self, object: Object, posn: usize) {
        self.establish(posn);
        self[posn].floor = Some(object);
    }

    pub fn has_object(&self, posn: usize) -> bool {
        posn < self.len() && self[posn].object.is_some()
    }

    pub fn has_monster(&self, posn: usize) -> bool {
        matches!(self[posn].object, Some(Monster(_)))
    }

    pub fn has_player(&self, posn: usize) -> bool {
        matches!(self[posn].object, Some(Player(_)))
    }

    pub fn render(&self, left: usize, right: usize) -> Vec<char> {
        let cright = right.min(self.len());
        let mut result: Vec<char> = self.0[left..cright]
            .iter()
            .map(|loc| loc.render())
            .collect();
        result.resize(right - left, ' ');
        result
    }

    pub fn len(&self) -> usize {
        self.0.len()
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
        let field = vec![Loc {
            object: Some(Rock),
            floor: None,
        }];
        let mut field = Field(field);
        field.insert_floor(Door, DOOR_POSN);
        field
    }
}
