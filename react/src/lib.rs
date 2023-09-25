use std::{collections::BTreeMap, cell::RefCell};

/// `InputCellId` is a unique identifier for an input cell.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct InputCellId(usize);
/// `ComputeCellId` is a unique identifier for a compute cell.
/// Values of type `InputCellId` and `ComputeCellId` should not be mutually assignable,
/// demonstrated by the following tests:
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input: react::ComputeCellId = r.create_input(111);
/// ```
///
/// ```compile_fail
/// let mut r = react::Reactor::new();
/// let input = r.create_input(111);
/// let compute: react::InputCellId = r.create_compute(&[react::CellId::Input(input)], |_| 222).unwrap();
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct ComputeCellId(usize);
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Hash, PartialOrd, Ord)]
pub struct CallbackId(usize);

trait NextId {
    fn next(&self) -> Self;
}

impl NextId for InputCellId {
    fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl NextId for ComputeCellId {
    fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

impl NextId for CallbackId {
    fn next(&self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CellId {
    Input(InputCellId),
    Compute(ComputeCellId),
}

pub struct InputCell<T> {
    value: T,
}

pub struct ComputeCell<'a, T> {
    value: T,
    dependencies: Vec<CellId>,
    func: Box<dyn Fn(&[T]) -> T + 'a>,
    callbacks: Table<CallbackId, Box<dyn FnMut(T) + 'a>>,
}

pub enum Cell<'a, T> {
    Input(InputCell<T>),
    Compute(ComputeCell<'a, T>),
}

struct Table<K, T> {
    data: BTreeMap<K, T>,
    index: K,
}

impl<K: NextId + Default + std::hash::Hash + Eq + Copy + Ord, T> Table<K, T> {
    fn new() -> Self {
        Table {
            data: BTreeMap::new(),
            index: K::default(),
        }
    }

    fn insert(&mut self, elem: T) -> K {
        let index = self.index;
        self.data.insert(index, elem);
        self.index = self.index.next();
        index
    }

    fn get(&self, index: K) -> Option<&T> {
        self.data.get(&index)
    }

    fn get_mut(&mut self, index: K) -> Option<&mut T> {
        self.data.get_mut(&index)
    }

    fn remove(&mut self, index: K) -> Option<T> {
        self.data.remove(&index)
    }

    fn iter_mut(&mut self) -> impl Iterator<Item = &mut T> {
        self.data.values_mut()
    }

    fn iter(&self) -> impl Iterator<Item = &T> {
        self.data.values()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum RemoveCallbackError {
    NonexistentCell,
    NonexistentCallback,
}

pub struct Reactor<'a, T> {
    inputs: Table<InputCellId, InputCell<T>>,
    computes: Table<ComputeCellId, RefCell<ComputeCell<'a, T>>>,
}

// You are guaranteed that Reactor will only be tested against types that are Copy + PartialEq.
impl<'a, T: Copy + PartialEq> Reactor<'a, T> {
    pub fn new() -> Self {
        Reactor {
            inputs: Table::new(),
            computes: Table::new(),
        }
    }

    // Creates an input cell with the specified initial value, returning its ID.
    pub fn create_input(&mut self, initial: T) -> InputCellId {
        self.inputs.insert(InputCell { value: initial })
    }

    // Creates a compute cell with the specified dependencies and compute function.
    // The compute function is expected to take in its arguments in the same order as specified in
    // `dependencies`.
    // You do not need to reject compute functions that expect more arguments than there are
    // dependencies (how would you check for this, anyway?).
    //
    // If any dependency doesn't exist, returns an Err with that nonexistent dependency.
    // (If multiple dependencies do not exist, exactly which one is returned is not defined and
    // will not be tested)
    //
    // Notice that there is no way to *remove* a cell.
    // This means that you may assume, without checking, that if the dependencies exist at creation
    // time they will continue to exist as long as the Reactor exists.
    pub fn create_compute<F: Fn(&[T]) -> T + 'a>(
        &mut self,
        dependencies: &[CellId],
        compute_func: F,
    ) -> Result<ComputeCellId, CellId> {
        let dep_values: Result<Vec<T>, CellId> = dependencies
            .iter()
            .map(|&cell| self.value(cell).ok_or(cell) )
            .collect();

        let value = compute_func(dep_values?.as_slice());

        let cell = ComputeCell {
            func: Box::new(compute_func),
            callbacks: Table::new(),
            dependencies: dependencies.to_vec(),
            value,
        };

        let id = self.computes.insert(RefCell::new(cell));

        Ok(id)
    }

    // Retrieves the current value of the cell, or None if the cell does not exist.
    //
    // You may wonder whether it is possible to implement `get(&self, id: CellId) -> Option<&Cell>`
    // and have a `value(&self)` method on `Cell`.
    //
    // It turns out this introduces a significant amount of extra complexity to this exercise.
    // We chose not to cover this here, since this exercise is probably enough work as-is.
    pub fn value(&self, id: CellId) -> Option<T> {
        match id {
            CellId::Input(id) => self.inputs.get(id).map(|cell| cell.value),
            CellId::Compute(id) => self.computes.get(id).map(|cell| cell.borrow().value),
        }
    }

    // update all compute cells
    fn react(&mut self) {

        for cell in self.computes.iter() {

            let dep_values: Vec<T> = cell.borrow().dependencies
                .iter()
                .filter_map(|&c| self.value(c)) 
                .collect();

            let mut cell = cell.borrow_mut();
            let value = (cell.func)(dep_values.as_slice());
            if value != cell.value {
                cell.value = value;
                // call cell callbacks
                for cb in cell.callbacks.iter_mut() {
                    cb(value)
                }
            }
        }
    }

    // Sets the value of the specified input cell.
    //
    // Returns false if the cell does not exist.
    //
    // Similarly, you may wonder about `get_mut(&mut self, id: CellId) -> Option<&mut Cell>`, with
    // a `set_value(&mut self, new_value: T)` method on `Cell`.
    //
    // As before, that turned out to add too much extra complexity.
    pub fn set_value(&mut self, id: InputCellId, new_value: T) -> bool {
        match self.inputs.get_mut(id) {
            Some(InputCell { value: v }) => {
                *v = new_value;

                self.react();

                true
            }
            _ => false,
        }
    }

    // Adds a callback to the specified compute cell.
    //
    // Returns the ID of the just-added callback, or None if the cell doesn't exist.
    //
    // Callbacks on input cells will not be tested.
    //
    // The semantics of callbacks (as will be tested):
    // For a single set_value call, each compute cell's callbacks should each be called:
    // * Zero times if the compute cell's value did not change as a result of the set_value call.
    // * Exactly once if the compute cell's value changed as a result of the set_value call.
    //   The value passed to the callback should be the final value of the compute cell after the
    //   set_value call.
    pub fn add_callback<F: FnMut(T) + 'a>(
        &mut self,
        id: ComputeCellId,
        callback: F,
    ) -> Option<CallbackId> {
        if let Some(cell) = self.computes.get(id) {
            return Some(cell.borrow_mut().callbacks.insert(Box::new(callback)));
        }

        None
    }

    // Removes the specified callback, using an ID returned from add_callback.
    //
    // Returns an Err if either the cell or callback does not exist.
    //
    // A removed callback should no longer be called.
    pub fn remove_callback(
        &mut self,
        cell: ComputeCellId,
        callback: CallbackId,
    ) -> Result<(), RemoveCallbackError> {
        match self.computes.get(cell) {
            Some(c) => match c.borrow_mut().callbacks.remove(callback) {
                Some(_) => Ok(()),
                _ => Err(RemoveCallbackError::NonexistentCallback),
            },
            _ => Err(RemoveCallbackError::NonexistentCell),
        }
    }
}
