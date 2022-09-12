use halo2_proofs::circuit::Value;
use halo2_proofs::{
    arithmetic::FieldExt,
    circuit::{Cell, Chip, Layouter, SimpleFloorPlanner},
    plonk::{Advice, Assigned, Circuit as PlonkCircuit, Column, ConstraintSystem, Error, Fixed, Instance},
    poly::Rotation,
};
use std::marker::PhantomData;
use halo2_proofs::plonk::FloorPlanner;
use pairing::group::ff::Field;

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

#[derive(Default)]
struct TutorialCircuit<F: FieldExt> {
    x: Value<F>,
    y: Value<F>,
    constant: F
}

pub trait Circuit <F: Field> {
    type Config: Clone;

    type FloorPlanner: FloorPlanner;

    fn without_witness(&self) -> Self;

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config;

    fn synthesize(&self, config: Self::Config, layouter: impl Layouter<F>) -> Result<(), Error>;
}

impl <F: FieldExt> Circuit<F> for TutorialCircuit<F> {
    type Config = TutorialConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witness(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let l = meta.advice_column();
        let r = meta.advice_column();
        let o = meta.advice_column();

        meta.enable_equality(l);
        meta.enable_equality(r);
        meta.enable_equality(o);

        let sm = meta.fixed_column();
        let sl = meta.fixed_column();
        let sr = meta.fixed_column();
        let so = meta.fixed_column();
        let sc = meta.fixed_column();
        let sp = meta.fixed_column();

        #[allow(non_snake_case)]
        let PI = meta.instance_column();
        meta.enable_equality(PI);

        meta.create_gate("mimi plonk", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            let r = meta.query_advice(r, Rotation::cur());
            let o = meta.query_advice(o, Rotation::cur());

            let sl = meta.query_fixed(sl, Rotation::cur());
            let sr = meta.query_fixed(sr, Rotation::cur());
            let so = meta.query_fixed(so, Rotation::cur());
            let sc = meta.query_fixed(sc, Rotation::cur());

            vec![l.clone() * sl + r.clone() * sr + l * r * sm + (o * so (-F::one())) + sc]
        });

        meta.create_gate("public input", |meta| {
            let l = meta.query_advice(l, Rotation::cur());
            #[allow(non_snake_case)]
            let PI = meta.query_instance(PI, Rotation::cur());
            let sp = meta.query_fixed(sp, Rotation::cur());
            vec![sp * (l -PI)]
        });

        TutorialConfig {
            l,
            r,
            o,
            sr,
            sl,
            so,
            sm,
            sc,
            PI
        }

    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let cs = TutorialChip::new(config);

        let x: Value<Assigned<_>> = self.x.into();
        let y: Value<Assigned<_>> = self.y.into();
        let consty = Assigned::from(self.constant);
        
        let (a0, b0, c0) = cs.raw_multiply(&mut layouter, || x.map(|x|(x, x, x**x)))?;

        let (a1, b1, c1) = cs.raw_multiply(&mut layouter, || y.map(|y|(y, y, y**y)))?;


    }
}

fn main() {
    println!("Hello, world!");
}
