use std::collections::HashMap;

use thiserror::Error;
use tracing::debug;

use crate::{
    logging_helpers::Targets,
    types::{Data, Vid},
};

use super::{BinaryOp, Instruction, Operation, Program, UnaryOp};

pub struct MockProcessor {}

fn map_zip<T, const N: usize, F>(a: &[T; N], b: &[T; N], f: F) -> [T; N]
where
    T: Clone,
    F: Fn(T, T) -> T,
{
    let mut result = a.clone();
    for (r, b_item) in result.iter_mut().zip(b.iter()) {
        *r = f(r.clone(), b_item.clone());
    }
    result
}

#[derive(Error, Debug)]
pub enum Error {
    #[error("No data with specified id is found")]
    DataNotFound,
}

impl MockProcessor {
    fn calculate(operation: &Operation<Data>) -> Data {
        let array = match operation {
            Operation::Sub(operation) => map_zip(
                operation.first.as_inner(),
                operation.second.as_inner(),
                reed_solomon_erasure::galois_8::add,
            ),
            Operation::Plus(operation) => map_zip(
                operation.first.as_inner(),
                operation.second.as_inner(),
                reed_solomon_erasure::galois_8::add,
            ),
            // inverses in GF(2^8) are the same values, because
            // the arithmetic is done on polynomials over GF(2)
            // and addition of any coefficient on itself gives 0
            // in GF(2)
            Operation::Inv(operation) => operation.operand.as_inner().map(|n| n),
            Operation::Nand(operation) => map_zip(
                operation.first.as_inner(),
                operation.second.as_inner(),
                |a, b| if a == 0 || b == 0 { 1 } else { 0 }, // NAND logic (0: false, 1: true)
            ),
            Operation::Nor(operation) => map_zip(
                operation.first.as_inner(),
                operation.second.as_inner(),
                |a, b| if a == 0 && b == 0 { 1 } else { 0 }, // NOR logic (0: false, 1: true)
            ),
        };
        Data(array)
    }

    fn retrieve_operand(operand: Vid, data_storage: &HashMap<Vid, Data>) -> Result<Data, Error> {
        data_storage
            .get(&operand)
            .cloned()
            .ok_or(Error::DataNotFound)
    }

    fn retrieve_binary(
        binary: BinaryOp<Vid>,
        data_storage: &HashMap<Vid, Data>,
    ) -> Result<BinaryOp<Data>, Error> {
        let BinaryOp { first, second } = binary;
        let first = Self::retrieve_operand(first, data_storage)?;
        let second = Self::retrieve_operand(second, data_storage)?;
        Ok(BinaryOp { first, second })
    }

    fn retrieve_unary(
        unary: UnaryOp<Vid>,
        data_storage: &HashMap<Vid, Data>,
    ) -> Result<UnaryOp<Data>, Error> {
        let operand = Self::retrieve_operand(unary.operand, data_storage)?;
        Ok(UnaryOp { operand })
    }

    fn retrieve_operands(
        op: Operation<Vid>,
        data_storage: &mut HashMap<Vid, Data>,
    ) -> Result<Operation<Data>, Error> {
        let retrieved = match op {
            Operation::Sub(binary) => Operation::Sub(Self::retrieve_binary(binary, data_storage)?),
            Operation::Plus(binary) => {
                Operation::Plus(Self::retrieve_binary(binary, data_storage)?)
            }
            Operation::Inv(unary) => Operation::Inv(Self::retrieve_unary(unary, data_storage)?),
            Operation::Nand(binary) => Operation::Nand(Self::retrieve_binary(binary, data_storage)?),
            Operation::Nor(binary) => Operation::Nor(Self::retrieve_binary(binary, data_storage)?),
        };
        Ok(retrieved)
    }

    pub fn execute_on(
        program: Program,
        data_storage: &mut HashMap<Vid, Data>,
    ) -> Result<(), Error> {
        let program_id = program.identifier.clone();
        debug!(target: Targets::ProgramExecution.into_str(), "(mock) Starting execution of program {:?}", program_id);
        for Instruction { operation, result } in program.instructions {
            let operation = Self::retrieve_operands(operation, data_storage)?;
            let result_value = Self::calculate(&operation);
            data_storage.insert(result, result_value);
        }
        debug!(target: Targets::ProgramExecution.into_str(), "(mock) Saving results of execution of program {:?}", program_id);
        Ok(())
    }
}
