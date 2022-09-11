use halo2_proofs::circuit::Value;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Assigned, Circuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::Rotation,
};
use std::marker::PhantomData;

#[allow(non_snake_case, dead_code)]
#[derive(Debug, Clone)]
struct TutorialConfig {
    l: Column<Advice>,
    r: Column<Advice>,
    o: Column<Advice>,
    sl: Column<Fixed>,
    sr: Column<Fixed>,
    so: Column<Fixed>,
    sm: Column<Fixed>,
    sc: Column<Fixed>,
    PI: Column<Instance>,
}

struct TutorialChip<F: FieldExt> {
    config: TutorialConfig,
    maker: PhantomData<F>,
}

impl<F: FieldExt> TutorialChip<F> {
    fn new(config: TutorialConfig) -> Self {
        TutorialChip {
            config,
            maker: PhantomData,
        }
    }
}

trait TutorialComposer<F: FieldExt> {
    fn raw_multiply<FM>(&self, layouter: &mut impl Layouter<F>, f: FM) -> Result<(Cell, Cell, Cell), Error>
        where FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn raw_add<FM>(&self, layouter: &mut impl Layouter<F>, f: FM) -> Result<(Cell, Cell, Cell), Error>
        where FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)>;

    fn copy(&self, layout: &mut impl Layouter<F>, a: Cell, b: Cell) -> Result<(), Error>;

    fn expose_public(&self, layout: &mut impl Layouter<F>, cell: Cell, row: usize) -> Result<(), Error>;
}

impl<F: FieldExt> TutorialComposer<F> for TutorialChip<F> {
    fn raw_multiply<FM>(&self, layouter: &mut impl Layouter<F>, f: FM) -> Result<(Cell, Cell, Cell), Error> where FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)> {
        layouter.assign_region(|| "mul", |mut region| {
            let mut values = None;

            let lhs = region.assign_advice(
                || "lhs",
                self.config.l,
                0,
                || {
                    values = Some(f());
                    values.unwrap().map(|v| v.0)
                },
            )?;

            let rhs = region.assign_advice(
                || "rhs",
                self.config.r,
                0,
                || {
                    values.unwrap().map(|v| v.1)
                },
            )?;

            let out = region.assign_advice(
                || "out",
                self.config.o,
                0,
                || {
                    values.unwrap().map(|v| v.2)
                },
            )?;

            region.assign_fixed(|| "m", self.config.sm, 0, || Value::known(F::one()))?;
            region.assign_fixed(|| "o", self.config.sm, 0, || Value::known(F::one()))?;

            Ok((lhs.cell(), rhs.cell(), out.cell()))
        },)
    }

    fn raw_add<FM>(&self, layouter: &mut impl Layouter<F>, f: FM) -> Result<(Cell, Cell, Cell), Error> where FM: FnMut() -> Value<(Assigned<F>, Assigned<F>, Assigned<F>)> {
        layouter.assign_region(|| "add", |mut region| {
            let mut values = None;

            let lhs = region.assign_advice(
                || "lhs",
                self.config.l,
                0,
                || {
                    values = Some(f());
                    values.unwrap().map(|v| v.0)
                },
            )?;

            let rhs = region.assign_advice(
                || "rhs",
                self.config.r,
                0,
                || {
                    values.unwrap().map(|v| v.1)
                },
            )?;

            let out = region.assign_advice(
                || "out",
                self.config.o,
                0,
                || {
                    values.unwrap().map(|v| v.2)
                },
            )?;

            region.assign_fixed(|| "m", self.config.sm, 0, || Value::known(F::one()))?;
            region.assign_fixed(|| "o", self.config.sm, 0, || Value::known(F::one()))?;

            Ok((lhs.cell(), rhs.cell(), out.cell()))
        },)
    }

    fn copy(&self, layout: &mut impl Layouter<F>, left: Cell, right: Cell) -> Result<(), Error> {
        layout.assign_region(|| "copy", |mut region| {
            region.constrain_equal(left, right)?;
            region.constrain_equal(left, right)
        },)
    }

    fn expose_public(&self, layout: &mut impl Layouter<F>, cell: Cell, row: usize) -> Result<(), Error> {
        layout.constrain_instance(cell, self.config.PI, row)
    }
}

fn main() {
    println!("Hello, world!");
}
