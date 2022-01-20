use std::iter::Map;
use std::ops::Index;
use itertools::Itertools;
use solana_program::program_error::ProgramError;
use solana_program::program_pack::{Pack, Sealed};
use std::marker::PhantomData;
use bitvec::prelude::*;
use bitvec::slice::IterOnes;

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct OptArrayRef<A> {
    id: usize,
    item_type: PhantomData<A>
}

impl<A> OptArrayRef<A> {
    fn new(id: usize) -> Self {
        Self { id, item_type: PhantomData }
    }
}

#[derive(Debug, Clone)]
pub struct OptArray<A, const SIZE: usize> {
    array: Box<[Option<A>; SIZE]>,
    free_slots: Vec<OptArrayRef<A>>
}

impl<A, const SIZE: usize> Index<OptArrayRef<A>> for OptArray<A, SIZE> {
    type Output = Option<A>;

    fn index(&self, r: OptArrayRef<A>) -> &Self::Output {
        &self.array[r.id]
    }
}

impl<A: Copy + PartialEq, const SIZE: usize> OptArray<A, SIZE> {
    pub const FLAGS_STORAGE_SIZE: usize = bitvec::mem::elts::<u8>(SIZE);

    pub fn from_vec(vec: Vec<Option<A>>) -> OptArray<A, SIZE> {
        let array = unsafe {
            // convert vector into a boxed array with static size
            Box::from_raw(Box::into_raw(vec.into_boxed_slice()) as *mut [Option<A>; SIZE])
        };
        let free_slots = array.iter().positions(|it| it.is_none()).map(OptArrayRef::new).collect_vec();

        OptArray { array, free_slots }
    }

    pub fn has_capacity(&self, capacity: usize) -> bool {
        self.free_slots.len() >= capacity
    }

    pub fn insert_many(&mut self, add_items: &Vec<A>) {
        if !self.has_capacity(add_items.len()) {
            panic!("Not enough free slots");
        }

        for item in add_items {
            let slot = self.free_slots.pop().unwrap();
            self.array[slot.id] = Some(*item);
        }
    }

    pub fn remove_by_refs(&mut self, refs: &Vec<OptArrayRef<A>>) {
        for r in refs {
            self.array[r.id] = None;
            self.free_slots.push(*r);
        }
    }

    pub fn find_ref(&self, item: &A) -> Option<OptArrayRef<A>> {
        self.array
            .iter()
            .position(|it| it == &Some(*item))
            .map(OptArrayRef::new)
    }

    pub fn find_refs(&self, items: &Vec<A>) -> Vec<OptArrayRef<A>> {
        return self.array
            .iter()
            .positions(|item_opt| item_opt.is_some() && items.contains(&item_opt.unwrap()))
            .map(OptArrayRef::new)
            .collect_vec();
    }
}

impl<A, const SIZE: usize> Sealed for OptArray<A, SIZE> {}

impl<A: Pack + Copy + PartialEq, const SIZE: usize> Pack for OptArray<A, SIZE> {
    const LEN: usize = SIZE * A::LEN;

    fn pack_into_slice(&self, dst: &mut [u8]) {
        dst.fill(0);
        for (i, chunk) in dst.chunks_exact_mut(A::LEN).enumerate() {
            for item in self.array[i].as_ref() {
                item.pack_into_slice(chunk);
            }
        }
    }

    fn unpack_from_slice(src: &[u8]) -> Result<Self, ProgramError> {
        let mut vec = Vec::with_capacity(SIZE);

        for chunk in src.chunks_exact(A::LEN) {
            vec.push(
                if chunk.iter().all(|&b| b == 0) {
                    None
                } else {
                    Some(A::unpack_from_slice(chunk)?)
                }
            );
        }

        Ok(OptArray::from_vec(vec))
    }
}

#[derive(Debug, Clone)]
pub struct OptArrayFlags<A, const STORAGE_SIZE: usize> {
    bit_arr: BitArray<[u8; STORAGE_SIZE]>,
    item_type: PhantomData<A>
}

impl<A, const STORAGE_SIZE: usize> OptArrayFlags<A, STORAGE_SIZE> {
    pub const STORAGE_SIZE: usize = STORAGE_SIZE;

    pub fn new(data: [u8; STORAGE_SIZE]) -> Self {
        Self { bit_arr: BitArray::new(data), item_type: PhantomData }
    }

    pub fn zero() -> Self {
        Self::new([0; STORAGE_SIZE])
    }

    pub fn enable(&mut self, r: OptArrayRef<A>) {
        self.bit_arr.set(r.id, true);
    }

    pub fn disable(&mut self, r: OptArrayRef<A>) {
        self.bit_arr.set(r.id, false);
    }

    pub fn is_enabled(&self, r: OptArrayRef<A>) -> bool {
        self.bit_arr[r.id]
    }

    pub fn count_enabled(&self) -> usize {
        self.bit_arr.count_ones()
    }

    pub fn iter_enabled(&self) -> Map<IterOnes<u8, Lsb0>, fn(usize) -> OptArrayRef<A>> {
        self.bit_arr.iter_ones().map(OptArrayRef::<A>::new)
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.bit_arr.as_raw_slice()
    }
}