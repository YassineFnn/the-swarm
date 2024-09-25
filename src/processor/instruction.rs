use serde::{Deserialize, Serialize};

macro_rules! impl_binary_constructor {
    ($function_name: ident, $enum_variant: ident) => {
        impl<TOperand, TResult> Instruction<TOperand, TResult> {
            pub fn $function_name(first: TOperand, second: TOperand, result: TResult) -> Self {
                Instruction {
                    operation: Operation::$enum_variant(BinaryOp { first, second }),
                    result,
                }
            }
        }
    };
}

macro_rules! impl_unary {
    ($function_name: ident, $enum_variant: ident) => {
        impl<TOperand, TResult> Instruction<TOperand, TResult> {
            pub fn $function_name(operand: TOperand, result: TResult) -> Self {
                Instruction {
                    operation: Operation::$enum_variant(UnaryOp { operand }),
                    result,
                }
            }
        }
    };
}

// todo: add instruction to save some result. will include distribution n stuff.
// is needed actually???? with proper reed&solomon, likely no
#[derive(Serialize, Deserialize, PartialEq, Eq, std::hash::Hash, Debug, Clone)]
pub struct Instruction<TOperand, TResult> {
    pub operation: Operation<TOperand>,
    pub result: TResult,
}

impl<TOperand, TResult> Instruction<TOperand, TResult> {
    // might be useful
    #[allow(unused)]
    pub fn map_operands<F, TNewOperand>(self, f: F) -> Instruction<TNewOperand, TResult>
    where
        F: Fn(TOperand) -> TNewOperand,
    {
        let new_op = match self.operation {
            Operation::Sub(binary) => Operation::Sub(BinaryOp {
                first: f(binary.first),
                second: f(binary.second),
            }),
            Operation::Plus(binary) => Operation::Plus(BinaryOp {
                first: f(binary.first),
                second: f(binary.second),
            }),
            Operation::Inv(unary) => Operation::Inv(UnaryOp {
                operand: f(unary.operand),
            }),
            Operation::Nand(binary) => Operation::Nand(BinaryOp { // Added Nand handling
                first: f(binary.first),
                second: f(binary.second),
            }),
            Operation::Nor(binary) => Operation::Nor(BinaryOp {    // Add case for NOR
                first: f(binary.first),
                second: f(binary.second),
            }),
        };
        Instruction {
            operation: new_op,
            result: self.result,
        }
    }

    // might be useful
    #[allow(unused)]
    pub fn as_ref(&self) -> Instruction<&TOperand, &TResult> {
        let ref_op = match self.operation {
            Operation::Sub(ref o) => Operation::Sub(o.as_ref()),
            Operation::Plus(ref o) => Operation::Plus(o.as_ref()),
            Operation::Inv(ref o) => Operation::Inv(o.as_ref()),
            Operation::Nand(ref o) => Operation::Nand(o.as_ref()), // Added Nand handling
            Operation::Nor(ref o) => Operation::Nor(o.as_ref()), // Added Nor handling
        };
        Instruction {
            operation: ref_op,
            result: &self.result,
        }
    }
}

impl<O, R> Instruction<Option<O>, R> {
    // might be useful
    #[allow(unused)]
    pub fn transpose_operation(self) -> Option<Instruction<O, R>> {
        match self.operation {
            Operation::Sub(o) => o.transpose().map(|o| Instruction {
                operation: Operation::Sub(o),
                result: self.result,
            }),
            Operation::Plus(o) => o.transpose().map(|o| Instruction {
                operation: Operation::Plus(o),
                result: self.result,
            }),
            Operation::Inv(o) => o.transpose().map(|o| Instruction {
                operation: Operation::Inv(o),
                result: self.result,
            }),
            Operation::Nand(o) => o.transpose().map(|o| Instruction {    // Add case for Nand
                operation: Operation::Nand(o),
                result: self.result,
            }),
            Operation::Nor(o) => o.transpose().map(|o| Instruction {     // Add case for Nor
                operation: Operation::Nor(o),
                result: self.result,
            }),
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, std::hash::Hash, Debug, Clone)]
pub enum Operation<TOperand> {
    Sub(BinaryOp<TOperand>),
    Plus(BinaryOp<TOperand>),
    Inv(UnaryOp<TOperand>),
    Nand(BinaryOp<TOperand>), // Added Nand variant
    Nor(BinaryOp<TOperand>),  // Add Nor as a variant
}

impl<TOperand> Operation<TOperand> {
    pub fn args_as_list(&self) -> Vec<&TOperand> {
        match self {
            Operation::Sub(BinaryOp { first, second })
            | Operation::Plus(BinaryOp { first, second })
            | Operation::Nand(BinaryOp { first, second })
            | Operation::Nor(BinaryOp { first, second }) => vec![first, second],  // Add case for Nand & NOR
            Operation::Inv(UnaryOp { operand }) => vec![operand],
        }
    }
}

impl_binary_constructor!(sub, Sub);
impl_binary_constructor!(plus, Plus);
impl_unary!(inv, Inv);
impl_binary_constructor!(nand, Nand); // Added constructor for Nand
impl_binary_constructor!(nor, Nor); // Added constructor for Nor


#[derive(Serialize, Deserialize, PartialEq, Eq, std::hash::Hash, Debug, Clone)]
pub struct BinaryOp<TOperand> {
    pub first: TOperand,
    pub second: TOperand,
}

impl<O> BinaryOp<O> {
    pub fn as_ref(&self) -> BinaryOp<&O> {
        BinaryOp {
            first: &self.first,
            second: &self.second,
        }
    }
}

impl<O> BinaryOp<Option<O>> {
    pub fn transpose(self) -> Option<BinaryOp<O>> {
        match (self.first, self.second) {
            (Some(first), Some(second)) => Some(BinaryOp { first, second }),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, PartialEq, Eq, std::hash::Hash, Debug, Clone)]
pub struct UnaryOp<TOperand> {
    pub operand: TOperand,
}

impl<O> UnaryOp<O> {
    pub fn as_ref(&self) -> UnaryOp<&O> {
        UnaryOp {
            operand: &self.operand,
        }
    }
}

impl<O> UnaryOp<Option<O>> {
    pub fn transpose(self) -> Option<UnaryOp<O>> {
        self.operand.map(|operand| UnaryOp { operand })
    }
}
