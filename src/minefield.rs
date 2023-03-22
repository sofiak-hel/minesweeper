use arrayvec::ArrayVec;

pub struct Coord<const W: usize, const H: usize>(pub usize, pub usize);

impl<const W: usize, const H: usize> Coord<W, H> {
    pub fn neighbours(&self) -> ArrayVec<Coord<W, H>, 8> {
        let mut list = ArrayVec::new();
        for y in -1..=1 {
            for x in -1..=1 {
                let (newx, newy) = (self.0 as i8 + x, self.1 as i8 + y);
                if newx >= 0 && newy >= 0 && newx < W as i8 && newy < H as i8 && (x != 0 || y != 0)
                {
                    list.push(Coord(newx as usize, newy as usize))
                }
            }
        }
        list
    }

    pub fn random() -> Coord<W, H> {
        Coord(rand::random::<usize>() % W, rand::random::<usize>() % H)
    }
}

#[derive(Debug)]
pub enum MinefieldError {
    InvalidCoordinate,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Cell {
    Empty,
    Label(u8),
    Flag,
    Hidden,
    Mine,
}

#[derive(Clone)]
pub struct Minefield<const W: usize, const H: usize> {
    mine_indices: [[bool; W]; H],
    pub field: [[Cell; W]; H],
}

impl<const W: usize, const H: usize> Minefield<W, H> {
    pub fn generate(mines: u8) -> Self {
        let mut mine_indices = [[false; W]; H];
        for _ in 0..mines {
            let mut coord = Coord::<W, H>::random();
            while mine_indices[coord.1][coord.0] {
                coord = Coord::random();
            }
            mine_indices[coord.1][coord.0] = true;
        }

        Minefield {
            mine_indices,
            field: [[Cell::Hidden; W]; H],
        }
    }

    pub fn is_game_over(&self) -> bool {
        self.field.into_iter().flatten().any(|c| c == Cell::Mine)
    }

    pub fn reveal(&mut self, coord: Coord<W, H>) -> Result<(), MinefieldError> {
        if coord.0 >= W || coord.1 >= H {
            Err(MinefieldError::InvalidCoordinate)
        } else {
            let field_cell = self.field[coord.1][coord.0];
            if field_cell == Cell::Flag || field_cell == Cell::Hidden {
                let cell = self.cell_contents(&coord);
                self.field[coord.1][coord.0] = cell;
                if cell == Cell::Empty {
                    for neighbor in coord.neighbours() {
                        self.reveal(neighbor)?;
                    }
                }
            }
            Ok(())
        }
    }

    pub fn flag(&mut self, coord: Coord<W, H>) -> Result<(), MinefieldError> {
        if coord.0 >= W || coord.1 >= H {
            Err(MinefieldError::InvalidCoordinate)
        } else {
            self.field[coord.1][coord.0] = match self.field[coord.1][coord.0] {
                Cell::Flag => Cell::Hidden,
                Cell::Hidden => Cell::Flag,
                c => c,
            };
            Ok(())
        }
    }

    fn cell_contents(&self, coord: &Coord<W, H>) -> Cell {
        if self.is_mine(coord) {
            Cell::Mine
        } else {
            let mines = coord
                .neighbours()
                .iter()
                .filter(|c| self.is_mine(c))
                .count() as u8;
            if mines == 0 {
                Cell::Empty
            } else {
                Cell::Label(mines)
            }
        }
    }

    fn is_mine(&self, coord: &Coord<W, H>) -> bool {
        self.mine_indices[coord.1][coord.0]
    }
}
