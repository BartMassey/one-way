// Copyright © 2019 Bart Massey
// [This program is licensed under the GPL version 3 or later.]
// Please see the file LICENSE in the source
// distribution of this software for license terms.

pub type Loc = char;

pub struct Field(Vec<Loc>);

impl Default for Field {
    fn default() -> Self {
        let field = Vec::new();
        Field(field)
    }
}
