use std::cmp::Ordering;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Parent {
    pub cell: usize,
    pub vector: usize,
}

pub struct DisjointedSet {
    parent: Vec<usize>,
}

impl DisjointedSet {
    pub fn new(size: usize) -> Self {
        let mut parent = Vec::with_capacity(size);

        for i in 0..size {
            parent.push(i);
        }

        DisjointedSet { parent }
    }

    pub fn find(&mut self, cell: usize) -> usize {
        if self.parent[cell] == cell {
            return cell;
        }

        let parent = self.find(self.parent[cell]);
        self.parent[cell] = parent;
        parent
    }

    pub fn union(&mut self, cell1: usize, cell2: usize) -> usize {
        let parent1 = self.find(cell1);
        let parent2 = self.find(cell2);

        self.parent[parent2] = parent1;
        parent1
    }
}

//This disjointed set works with two layers, effectively two disjointed sets in one,
// however those two disjointed sets communicate with each other, to achieve this we use an array
// that index both sets and this lets cells be parents of vector in another set
pub struct VectorDisjointedSet {
    parent: Vec<[Parent; 2]>,
    rank: Vec<[usize; 2]>,
}

impl VectorDisjointedSet {
    pub fn new(size: usize) -> Self {
        let mut parent = Vec::with_capacity(size);
        let mut rank = Vec::with_capacity(size);

        for _ in 0..size {
            parent.push([
                Parent {
                    cell: usize::MAX,
                    vector: usize::MAX,
                },
                Parent {
                    cell: usize::MAX,
                    vector: usize::MAX,
                },
            ]);
            rank.push([0, 0]);
        }

        VectorDisjointedSet { parent, rank }
    }

    pub fn exists(&self, cell: usize, vector: usize) -> bool {
        self.parent[cell][vector].cell != usize::MAX
    }

    pub fn init(&mut self, cell: usize, vector: usize) -> Parent {
        let parent = Parent { cell, vector };
        self.parent[cell][vector] = parent;
        self.rank[cell] = [0, 0];
        parent
    }

    // modified find function to work with two vector per cell
    // find with path compression
    pub fn find(&mut self, cell: usize, vector: usize) -> Parent {
        if self.parent[cell][vector].cell == usize::MAX || self.parent[cell][vector].cell == cell {
            return self.parent[cell][vector];
        }

        let result = self.find(
            self.parent[cell][vector].cell,
            self.parent[cell][vector].vector,
        );

        self.parent[cell][vector] = result;

        self.parent[cell][vector]
    }

    //union by rank
    pub fn union(&mut self, cell1: usize, vector1: usize, cell2: usize, vector2: usize) -> Parent {
        let parent1 = self.find(cell1, vector1);
        let parent2 = self.find(cell2, vector2);

        match self.rank[parent1.cell][parent1.vector].cmp(&self.rank[parent2.cell][parent2.vector])
        {
            Ordering::Less => {
                self.parent[parent1.cell][parent1.vector] = parent2;
                parent2
            }
            Ordering::Greater => {
                self.parent[parent2.cell][parent2.vector] = parent1;
                parent1
            }
            Ordering::Equal => {
                self.parent[parent2.cell][parent2.vector] = parent1;
                self.rank[parent1.cell][parent1.vector] += 1;
                parent1
            }
        }
    }
}
