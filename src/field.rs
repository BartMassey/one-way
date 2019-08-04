// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

use crate::*;

use std::ops::{Index, IndexMut};

pub enum Object {
    Hero(Player),
    Door,
}
use Object::*;

impl Object {
    pub fn render(&self) -> char {
        match self {
            Hero(_) => '@',
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
        self.object
            .as_ref()
            .and_then(|obj| Some(obj))
            .or_else(|| self.floor.as_ref())
    }
}

pub struct Field(Vec<Loc>);

impl Field {
    pub fn insert(&mut self, object: Object, posn: usize) {
        if self.0.len() < posn {
            self.0.resize_with(posn, Loc::default);
        }
        assert!(self.0[posn].object.is_none());
        self.0[posn].object = Some(object);
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
        field.push(Loc { object: Some(Door), floor: None });
        field.push(Loc::default());
        Field(field)
    }
}
