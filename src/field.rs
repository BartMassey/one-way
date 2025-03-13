// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

//! Model and maintain the game playfield.

use crate::*;

use std::ops::{Index, IndexMut};

/// Game object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Object {
    /// Rocks are immovable and inert. A rock blocks the
    /// near end.
    Rock,
    /// Monsters are by ID.
    Monster(u64),
    /// Players are by ID.
    Player(u64),
    /// The door is the exit square, at the
    /// far end.
    Door,
}
use Object::*;

impl Object {
    /// Get the display character associated with an object.
    pub fn render(&self) -> char {
        match self {
            Rock => '#',
            Monster(_) => 'M',
            Player(_) => '@',
            Door => '+',
        }
    }
}

/// Things that can be in a location.
#[derive(Default)]
pub struct Loc {
    /// Object in the location.
    pub object: Option<Object>,
    /// Floor for the location.
    pub floor: Option<Object>,
}

impl Loc {
    /// Get the topmost object at the location.
    pub fn top(&self) -> Option<&Object> {
        // XXX I think the closure is really needed, but
        // Clippy doesn't like it.
        #[allow(clippy::unnecessary_lazy_evaluations)]
        self.object.as_ref().or_else(|| self.floor.as_ref())
    }

    /// Get the display character for the object at the
    /// location, or '.' for an empty location.
    pub fn render(&self) -> char {
        match self.top() {
            Some(obj) => obj.render(),
            None => '.',
        }
    }
}

/// The playfield is a long vector of locations. It is
/// created lazily as needed, which would allow infinite
/// playfields.
pub struct Field(Vec<Loc>);

#[allow(clippy::len_without_is_empty)]
impl Field {
    /// Make sure that the given location exists by growing
    /// the vector as necessary.
    pub fn establish(&mut self, posn: usize) {
        if self.len() <= posn {
            self.0.resize_with(posn + 1, Loc::default);
        }
    }

    /// Insert the given object at the given position.
    pub fn insert(&mut self, object: Object, posn: usize) {
        self.establish(posn);
        self[posn].object = Some(object);
    }

    /// Insert the given floor object at the given position.
    pub fn insert_floor(&mut self, object: Object, posn: usize) {
        self.establish(posn);
        self[posn].floor = Some(object);
    }

    /// Does the position have a (non-floor) object?
    pub fn has_object(&self, posn: usize) -> bool {
        posn < self.len() && self[posn].object.is_some()
    }

    /// Does the position have a monster?
    pub fn has_monster(&self, posn: usize) -> bool {
        matches!(self[posn].object, Some(Monster(_)))
    }

    /// Does the position have a player avatar?
    pub fn has_player(&self, posn: usize) -> bool {
        matches!(self[posn].object, Some(Player(_)))
    }

    /// Get the render chars for the given span. Note that
    /// not-yet-created locations will be blank.
    pub fn render(&self, left: usize, right: usize) -> Vec<char> {
        let cright = right.min(self.len());
        let mut result: Vec<char> = self.0[left..cright]
            .iter()
            .map(|loc| loc.render())
            .collect();
        result.resize(right - left, ' ');
        result
    }

    /// How much of the playfield is currently live?
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
