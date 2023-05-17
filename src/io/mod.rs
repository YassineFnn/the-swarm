//! Reading & parsing initial demo input.

use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{error::Error, fmt::Display, path::Path};

use crate::{
    processor::{Instruction, Instructions},
    types::{Data, Shard, Vid},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct InputData {
    pub data: Vec<(Vid, Data)>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InputProgram {
    pub instructions: Instructions,
}

pub async fn read_input<P, T>(path: P) -> anyhow::Result<T>
where
    P: AsRef<Path>,
    T: DeserializeOwned,
{
    let raw = tokio::fs::read_to_string(&path).await?;
    let data = serde_json::from_str::<T>(&raw)?;
    Ok(data)
}

#[allow(dead_code)]
pub async fn write_input<P, T>(path: P, data: T) -> anyhow::Result<()>
where
    P: AsRef<Path>,
    T: Serialize,
{
    let raw = serde_json::to_string_pretty(&data)?;
    tokio::fs::write(path, raw).await?;
    Ok(())
}

#[allow(dead_code)]
/// Write some basic layout to path to see the format
/// for generating other inputs
pub async fn test_write_input<P>(path_data: P, path_program: P) -> anyhow::Result<()>
where
    P: AsRef<Path>,
{
    let test_data = InputData {
        data: vec![
            (Vid(1), [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]),
            (Vid(2), [37, 144, 123, 1, 0, 0, 0, 14, 53, 23, 1, 1]),
        ],
    };
    write_input(path_data, test_data).await?;

    let test_program = InputProgram {
        instructions: vec![
            Instruction::plus(Vid(1), Vid(2), Vid(3)),
            Instruction::dot(Vid(1), Vid(2), Vid(4)),
            Instruction::inv(Vid(4), Vid(5)),
        ],
    };
    write_input(path_program, test_program).await?;
    Ok(())
}
