//! Validador e planejador de fusão de grafos. Produz uma representação
//! intermediária textual; não gera código nativo nem executa kernels JIT.
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpType {
    MatrixMultiply,
    VectorAdd,
    ReLU,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GraphNode {
    pub id: usize,
    pub op: OpType,
    pub inputs: Vec<usize>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    UnsupportedOptimizationLevel(u8),
    DuplicateNodeId(usize),
    MissingInput { node: usize, input: usize },
    CycleOrNonTopologicalInput { node: usize, input: usize },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompiledInstruction {
    FusedMatMulRelu { matmul_node: usize, relu_node: usize },
    Single { node: usize, op: OpType },
}

#[derive(Debug, Clone, Copy)]
pub struct GraphJitCompiler {
    optimization_level: u8,
}

impl GraphJitCompiler {
    pub fn new(optimization_level: u8) -> Result<Self, CompileError> {
        if optimization_level > 3 {
            return Err(CompileError::UnsupportedOptimizationLevel(optimization_level));
        }
        Ok(Self { optimization_level })
    }

    pub fn optimization_level(&self) -> u8 {
        self.optimization_level
    }

    /// Valida as dependências em ordem topológica e combina MatMul + ReLU
    /// somente quando a ReLU realmente consome a saída da MatMul, sem ramificações.
    pub fn compile_and_fuse(&self, nodes: &[GraphNode]) -> Result<Vec<CompiledInstruction>, CompileError> {
        let mut seen = HashSet::new();
        let mut consumers: HashMap<usize, usize> = HashMap::new();
        for node in nodes {
            if !seen.insert(node.id) {
                return Err(CompileError::DuplicateNodeId(node.id));
            }
            for input in &node.inputs {
                *consumers.entry(*input).or_default() += 1;
            }
        }

        let mut available = HashSet::new();
        for node in nodes {
            for input in &node.inputs {
                if !seen.contains(input) {
                    return Err(CompileError::MissingInput { node: node.id, input: *input });
                }
                if !available.contains(input) {
                    return Err(CompileError::CycleOrNonTopologicalInput { node: node.id, input: *input });
                }
            }
            available.insert(node.id);
        }

        let mut output = Vec::with_capacity(nodes.len());
        let mut index = 0;
        while index < nodes.len() {
            let current = &nodes[index];
            let next = nodes.get(index + 1);
            let can_fuse = matches!(current.op, OpType::MatrixMultiply)
                && consumers.get(&current.id) == Some(&1)
                && next.is_some_and(|candidate| {
                    matches!(candidate.op, OpType::ReLU) && candidate.inputs.contains(&current.id)
                });
            if can_fuse {
                let relu = next.expect("can_fuse requires next node");
                output.push(CompiledInstruction::FusedMatMulRelu {
                    matmul_node: current.id,
                    relu_node: relu.id,
                });
                index += 2;
            } else {
                output.push(CompiledInstruction::Single { node: current.id, op: current.op.clone() });
                index += 1;
            }
        }
        Ok(output)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fuses_only_connected_unbranched_matmul_relu() {
        let compiler = GraphJitCompiler::new(3).unwrap();
        let graph = vec![
            GraphNode { id: 1, op: OpType::MatrixMultiply, inputs: vec![] },
            GraphNode { id: 2, op: OpType::ReLU, inputs: vec![1] },
            GraphNode { id: 3, op: OpType::VectorAdd, inputs: vec![2] },
        ];
        assert_eq!(compiler.compile_and_fuse(&graph).unwrap(), vec![
            CompiledInstruction::FusedMatMulRelu { matmul_node: 1, relu_node: 2 },
            CompiledInstruction::Single { node: 3, op: OpType::VectorAdd },
        ]);
    }

    #[test]
    fn rejects_missing_or_forward_inputs() {
        let compiler = GraphJitCompiler::new(1).unwrap();
        let graph = vec![GraphNode { id: 2, op: OpType::ReLU, inputs: vec![99] }];
        assert_eq!(compiler.compile_and_fuse(&graph), Err(CompileError::MissingInput { node: 2, input: 99 }));
    }

    #[test]
    fn does_not_fuse_a_branched_value() {
        let compiler = GraphJitCompiler::new(2).unwrap();
        let graph = vec![
            GraphNode { id: 1, op: OpType::MatrixMultiply, inputs: vec![] },
            GraphNode { id: 2, op: OpType::ReLU, inputs: vec![1] },
            GraphNode { id: 3, op: OpType::VectorAdd, inputs: vec![1] },
        ];
        let instructions = compiler.compile_and_fuse(&graph).unwrap();
        assert!(matches!(instructions[0], CompiledInstruction::Single { node: 1, .. }));
    }

    #[test]
    fn validates_optimization_level() {
        assert_eq!(GraphJitCompiler::new(4).unwrap_err(), CompileError::UnsupportedOptimizationLevel(4));
    }
}
